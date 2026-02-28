// commands/data_analysts.rs — Data Analyst + universal data request commands
// Source of truth: 02_DATA_ANALYSTS.md, 00_MASTER_GUIDE.md
//
// Use cases implemented:
// - UC-DRQ-01: Submit Data Request (any authenticated user)
// - UC-DA-01:  Receive Approved Data Request (Data Analyst inbox)
// - UC-DA-02:  Process Data Request (Data Analyst workbench)
// - UC-DA-03:  Submit Data Response (Data Analyst → Statistician outbound review)
// - UC-DA-04:  Access Company Data (permission enforcement — no standalone screen)
//
// The Statistician's gating commands (UC-STAT-01/02/03) remain in directors.rs
// but are updated to query the dedicated data_requests / data_responses tables.

use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::{
    audit::{write_audit_log, AuditOperation},
    error::AppError,
    state::{AppState, Role},
};

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SubmitDataRequestPayload {
    pub dataset_description: String,
    pub scope: String,
    pub purpose: String,
    pub urgency: String,
    pub sensitivity_note: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DataRequest {
    pub id: Uuid,
    pub requested_by: Uuid,
    pub requester_name: String,
    pub dataset_description: String,
    pub scope: String,
    pub purpose: String,
    pub urgency: String,
    pub sensitivity_note: Option<String>,
    pub status: String,
    pub statistician_decision_reason: Option<String>,
    pub assigned_analyst_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DataRequestBrief {
    pub id: Uuid,
    pub requester_name: String,
    pub dataset_description: String,
    pub scope: String,
    pub urgency: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessDataRequestPayload {
    pub request_id: Uuid,
    pub action: String, // 'start_processing'
}

#[derive(Debug, Deserialize)]
pub struct SubmitDataResponsePayload {
    pub request_id: Uuid,
    pub result_payload: Option<serde_json::Value>,
    pub analyst_notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DataResponse {
    pub id: Uuid,
    pub request_id: Uuid,
    pub prepared_by: Uuid,
    pub analyst_name: String,
    pub result_payload: Option<serde_json::Value>,
    pub spreadsheet_storage_path: Option<String>,
    pub status: String,
    pub statistician_review_note: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
}

// ── UC-DRQ-01: Submit Data Request ────────────────────────────────────────────

/// Submit a cross-department data request (any authenticated user).
///
/// Routes to The Statistician's queue (bypass authority — no Directors vote).
///
/// **Access:** Any authenticated user.
#[tauri::command]
pub async fn submit_data_request(
    state: State<'_, AppState>,
    payload: SubmitDataRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    // Validate urgency
    let valid_urgencies = ["low", "medium", "high", "critical"];
    if !valid_urgencies.contains(&payload.urgency.as_str()) {
        return Err(AppError::Internal(
            "Urgency must be one of: low, medium, high, critical.".into(),
        ));
    }

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO data_requests
          (requested_by, dataset_description, scope, purpose, urgency, sensitivity_note)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.dataset_description)
    .bind(&payload.scope)
    .bind(&payload.purpose)
    .bind(&payload.urgency)
    .bind(&payload.sensitivity_note)
    .fetch_one(&state.db_pool)
    .await?;

    let request_id = row.0;

    // Notify The Statistician — find user(s) with that role
    let statisticians: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'TheStatistician'
          AND u.deleted_at IS NULL
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (stat_id,) in &statisticians {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'data_request:submitted', $2::jsonb)
            "#,
        )
        .bind(stat_id)
        .bind(serde_json::json!({
            "request_id": request_id,
            "requester_name": user.full_name,
            "dataset_description": payload.dataset_description,
            "urgency": payload.urgency,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    // Invalidate Statistician queue cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del("data_requests:statistician_queue").await;
    }

    write_audit_log(
        &state.db_pool,
        "data_requests",
        request_id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "dataset_description": payload.dataset_description,
            "scope": payload.scope,
            "purpose": payload.purpose,
            "urgency": payload.urgency,
            "sensitivity_note": payload.sensitivity_note,
        })),
    )
    .await?;

    Ok(request_id)
}

// ── UC-DRQ-01 (tracker): Get My Data Requests ────────────────────────────────

/// Returns all data requests submitted by the current user (status tracker).
///
/// **Access:** Any authenticated user (sees own requests only).
#[tauri::command]
pub async fn get_my_data_requests(
    state: State<'_, AppState>,
) -> Result<Vec<DataRequestBrief>, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    let rows = sqlx::query_as::<_, DataRequestBrief>(
        r#"
        SELECT dr.id, u.full_name AS requester_name,
               dr.dataset_description, dr.scope, dr.urgency,
               dr.status, dr.created_at
        FROM data_requests dr
        JOIN users u ON u.id = dr.requested_by
        WHERE dr.requested_by = $1
          AND dr.deleted_at IS NULL
        ORDER BY dr.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows)
}

// ── Get single data request detail ────────────────────────────────────────────

/// Returns a single data request with full detail.
///
/// **Access:** The original requester, any DataAnalyst, TheStatistician, or Administrator.
#[tauri::command]
pub async fn get_data_request_detail(
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<DataRequest, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    let request = sqlx::query_as::<_, DataRequest>(
        r#"
        SELECT dr.id, dr.requested_by, u.full_name AS requester_name,
               dr.dataset_description, dr.scope, dr.purpose, dr.urgency,
               dr.sensitivity_note, dr.status, dr.statistician_decision_reason,
               dr.assigned_analyst_id, dr.created_at, dr.updated_at
        FROM data_requests dr
        JOIN users u ON u.id = dr.requested_by
        WHERE dr.id = $1 AND dr.deleted_at IS NULL
        "#,
    )
    .bind(request_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Data request not found.".into()))?;

    // Access: requester sees own, DataAnalyst sees assigned/approved, Statistician/Admin sees all
    let is_requester = request.requested_by == user.id;
    let is_analyst = user.role == Role::DataAnalyst;
    let is_stat_or_admin =
        user.role == Role::TheStatistician || user.role == Role::Administrator;

    if !is_requester && !is_analyst && !is_stat_or_admin {
        return Err(AppError::Forbidden);
    }

    Ok(request)
}

// ── UC-DA-01: Analyst Inbox ───────────────────────────────────────────────────

/// Returns approved data requests for the Data Analyst team.
///
/// **Access:** DataAnalyst or Administrator.
#[tauri::command]
pub async fn get_analyst_inbox(
    state: State<'_, AppState>,
) -> Result<Vec<DataRequestBrief>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::DataAnalyst, Role::Administrator]);

    // Check Redis cache first
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn
            .get::<_, String>("data_requests:analyst_inbox")
            .await
        {
            if let Ok(parsed) = serde_json::from_str::<Vec<DataRequestBrief>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let rows = sqlx::query_as::<_, DataRequestBrief>(
        r#"
        SELECT dr.id, u.full_name AS requester_name,
               dr.dataset_description, dr.scope, dr.urgency,
               dr.status, dr.created_at
        FROM data_requests dr
        JOIN users u ON u.id = dr.requested_by
        WHERE dr.status IN ('approved', 'processing')
          AND dr.deleted_at IS NULL
        ORDER BY
          CASE dr.urgency
            WHEN 'critical' THEN 0
            WHEN 'high' THEN 1
            WHEN 'medium' THEN 2
            WHEN 'low' THEN 3
          END,
          dr.created_at ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    // Cache for 5 minutes
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(serialized) = serde_json::to_string(&rows) {
            let _: Result<(), _> = conn
                .set_ex("data_requests:analyst_inbox", &serialized, 300)
                .await;
        }
    }

    Ok(rows)
}

// ── UC-DA-02: Process Data Request ────────────────────────────────────────────

/// Analyst starts processing a data request (status → 'processing').
///
/// **Access:** DataAnalyst or Administrator.
#[tauri::command]
pub async fn process_data_request(
    state: State<'_, AppState>,
    payload: ProcessDataRequestPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::DataAnalyst, Role::Administrator]);

    if payload.action != "start_processing" {
        return Err(AppError::Internal(
            "Action must be 'start_processing'.".into(),
        ));
    }

    // Check current status — only approved requests can be picked up
    let current: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM data_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    match current {
        Some((ref s,)) if s == "approved" => {}
        Some((ref s,)) => {
            return Err(AppError::Internal(format!(
                "Cannot process: request is '{}', expected 'approved'.",
                s
            )));
        }
        None => return Err(AppError::Internal("Data request not found.".into())),
    }

    sqlx::query(
        r#"
        UPDATE data_requests
        SET status = 'processing',
            assigned_analyst_id = $2,
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    // Invalidate caches
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del("data_requests:analyst_inbox").await;
        let _: Result<(), _> = conn
            .del(format!("data_request:status:{}", payload.request_id))
            .await;
    }

    write_audit_log(
        &state.db_pool,
        "data_requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": "approved" })),
        Some(serde_json::json!({
            "status": "processing",
            "assigned_analyst_id": user.id,
        })),
    )
    .await?;

    Ok(())
}

// ── UC-DA-03: Submit Data Response ────────────────────────────────────────────

/// Submit a processed data response, optionally with a spreadsheet file.
/// Routes to The Statistician for outbound sensitivity review.
///
/// **Access:** DataAnalyst or Administrator.
#[tauri::command]
pub async fn submit_data_response(
    state: State<'_, AppState>,
    payload: SubmitDataResponsePayload,
    file_bytes: Option<Vec<u8>>,
    filename: Option<String>,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::DataAnalyst, Role::Administrator]);

    // Verify the request exists and is in 'processing' status
    let current: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM data_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    match current {
        Some((ref s,)) if s == "processing" => {}
        Some((ref s,)) => {
            return Err(AppError::Internal(format!(
                "Cannot submit response: request is '{}', expected 'processing'.",
                s
            )));
        }
        None => return Err(AppError::Internal("Data request not found.".into())),
    }

    // Build result_payload JSONB
    let result_json = payload.result_payload.unwrap_or(serde_json::json!({}));

    // Insert response record first to get the ID for storage path
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO data_responses
          (request_id, prepared_by, result_payload)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(payload.request_id)
    .bind(user.id)
    .bind(&result_json)
    .fetch_one(&state.db_pool)
    .await?;

    let response_id = row.0;

    // Upload spreadsheet to Supabase Storage if provided
    let mut storage_path: Option<String> = None;
    if let (Some(bytes), Some(fname)) = (file_bytes, filename) {
        let path = format!("data-responses/{}/{}", response_id, fname);

        // Ensure bucket exists (idempotent)
        let client = reqwest::Client::new();
        let _ = client
            .post(&format!("{}/bucket", state.supabase_storage_url))
            .header(
                "Authorization",
                format!("Bearer {}", state.supabase_service_jwt),
            )
            .header("Content-Type", "application/json")
            .body(r#"{"id":"rusa-files","name":"rusa-files","public":false}"#)
            .send()
            .await;

        let upload_url = format!(
            "{}/object/rusa-files/{}",
            state.supabase_storage_url, path
        );

        let resp = client
            .post(&upload_url)
            .header(
                "Authorization",
                format!("Bearer {}", state.supabase_service_jwt),
            )
            .header(
                "Content-Type",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            )
            .body(bytes)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Storage upload failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Storage upload returned {}: {}",
                status, body
            )));
        }

        // Update the response record with the storage path
        sqlx::query(
            "UPDATE data_responses SET spreadsheet_storage_path = $2 WHERE id = $1",
        )
        .bind(response_id)
        .bind(&path)
        .execute(&state.db_pool)
        .await?;

        storage_path = Some(path);
    }

    // Update the data request status to pending_outbound_review
    sqlx::query(
        r#"
        UPDATE data_requests
        SET status = 'pending_outbound_review', updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .execute(&state.db_pool)
    .await?;

    // Notify Statistician about outbound response
    let statisticians: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'TheStatistician'
          AND u.deleted_at IS NULL
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (stat_id,) in &statisticians {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'data_response:ready_for_review', $2::jsonb)
            "#,
        )
        .bind(stat_id)
        .bind(serde_json::json!({
            "response_id": response_id,
            "request_id": payload.request_id,
            "analyst_name": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    // Invalidate caches
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del("data_requests:analyst_inbox").await;
        let _: Result<(), _> = conn
            .del(format!("data_request:status:{}", payload.request_id))
            .await;
        let _: Result<(), _> = conn.del("data_requests:statistician_queue").await;
    }

    write_audit_log(
        &state.db_pool,
        "data_responses",
        response_id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "request_id": payload.request_id,
            "has_spreadsheet": storage_path.is_some(),
            "storage_path": storage_path,
        })),
    )
    .await?;

    Ok(response_id)
}

// ── Get Data Response for a Request ───────────────────────────────────────────

/// Returns the data response(s) for a given request.
///
/// **Access:** The original requester (only if delivered), DataAnalyst, TheStatistician, Administrator.
#[tauri::command]
pub async fn get_data_response(
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<Option<DataResponse>, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    let response = sqlx::query_as::<_, DataResponse>(
        r#"
        SELECT dr.id, dr.request_id, dr.prepared_by,
               u.full_name AS analyst_name,
               dr.result_payload, dr.spreadsheet_storage_path,
               dr.status, dr.statistician_review_note,
               dr.submitted_at, dr.cleared_at, dr.delivered_at
        FROM data_responses dr
        JOIN users u ON u.id = dr.prepared_by
        WHERE dr.request_id = $1
        ORDER BY dr.submitted_at DESC
        LIMIT 1
        "#,
    )
    .bind(request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    // If the requester is viewing, only allow if the response is delivered
    if let Some(ref resp) = response {
        let is_requester = {
            let req_row: Option<(Uuid,)> = sqlx::query_as(
                "SELECT requested_by FROM data_requests WHERE id = $1",
            )
            .bind(request_id)
            .fetch_optional(&state.db_pool)
            .await?;
            req_row.map_or(false, |(rid,)| rid == user.id)
        };

        let is_analyst = user.role == Role::DataAnalyst;
        let is_stat_or_admin =
            user.role == Role::TheStatistician || user.role == Role::Administrator;

        if is_requester && resp.status != "cleared" && resp.delivered_at.is_none() {
            // Requester can only see cleared/delivered responses
            return Ok(None);
        }

        if !is_requester && !is_analyst && !is_stat_or_admin {
            return Err(AppError::Forbidden);
        }
    }

    Ok(response)
}

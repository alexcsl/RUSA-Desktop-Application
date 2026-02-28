// commands/astronauts.rs — Astronauts subsystem commands
// Source of truth: 05_ASTRONAUTS.md, 00_MASTER_GUIDE.md
//
// Use cases implemented:
// - UC-AS-01: Receive Mission Assignment (get_my_missions)
// - UC-AS-02: View Mission Document (get_mission_detail) — includes team counters
// - UC-AS-03: Submit Mission Status Report (submit_mission_status_report)
// - UC-AS-04: Submit Mission Completion Request (submit_mission_completion_request) — uploads evidence to Storage
// - UC-AS-05: View Personal Digital Journal / Mission Counter (get_personal_journal)
// - UC-AS-06: View Colleague Mission Counters (get_colleague_counters)
//
// All commands: authenticate, check role (Astronaut), execute, audit, invalidate Redis.

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

/// Summary of a mission for the inbox listing (UC-AS-01).
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MissionSummary {
    pub id: Uuid,
    pub title: String,
    #[sqlx(rename = "type")]
    pub mission_type: String,
    pub danger_level: Option<String>,
    pub location: String,
    pub status: String,
    pub assigned_by_name: String,
    pub created_at: DateTime<Utc>,
}

/// Full mission document with procedures and dangers (UC-AS-02).
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MissionDocument {
    pub id: Uuid,
    pub title: String,
    #[sqlx(rename = "type")]
    pub mission_type: String,
    pub danger_level: Option<String>,
    pub location: String,
    pub mission_objective: Option<String>,
    pub procedures: Option<String>,
    pub known_dangers: Option<String>,
    pub status: String,
    pub assigned_by: Uuid,
    pub assigned_by_name: String,
    pub created_at: DateTime<Utc>,
}

/// Team member with their mission counters (UC-AS-02 / UC-AS-06).
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TeamMemberCounter {
    pub astronaut_id: Uuid,
    pub full_name: String,
    pub interstellar_count: i32,
    pub terrain_count: i32,
}

/// Status report row (for listing reports on a mission detail page).
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct StatusReportItem {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub submitted_by: Uuid,
    pub submitter_name: String,
    pub report_date: DateTime<Utc>,
    pub month_tracker: Option<String>,
    pub rag_status: Option<String>,
    pub current_status: String,
    pub issues_blockers: Option<String>,
    pub collected_samples_last_month: Option<String>,
    pub progress_last_month: Option<String>,
    pub plans_next_month: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Payload for submitting a status report (UC-AS-03).
#[derive(Debug, Deserialize)]
pub struct SubmitStatusReportPayload {
    pub mission_id: Uuid,
    pub report_date: String,       // ISO-8601 date-time
    pub month_tracker: Option<String>,
    pub rag_status: Option<String>, // 'red' | 'amber' | 'green'
    pub current_status: String,
    pub issues_blockers: Option<String>,
    pub collected_samples_last_month: Option<String>,
    pub progress_last_month: Option<String>,
    pub plans_next_month: Option<String>,
}

/// Payload for submitting a completion request (UC-AS-04).
/// Evidence files are sent separately as file_bytes_list.
#[derive(Debug, Deserialize)]
pub struct SubmitCompletionRequestPayload {
    pub mission_id: Uuid,
    pub findings_summary: String,
    pub file_names: Vec<String>,       // original filenames
    pub content_types: Vec<String>,    // MIME types per file
}

/// Completion request row (viewable to astronaut).
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CompletionRequestItem {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub submitted_by: Uuid,
    pub findings_summary: String,
    pub evidence_storage_paths: Option<Vec<String>>,
    pub status: String,
    pub wanderer_note: Option<String>,
    pub taskmaster_note: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Personal journal: one completed mission entry (UC-AS-05).
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct JournalEntry {
    pub mission_id: Uuid,
    pub title: String,
    #[sqlx(rename = "type")]
    pub mission_type: String,
    pub location: String,
    pub danger_level: Option<String>,
    pub completed_at: DateTime<Utc>,
}

/// Full journal including counters (UC-AS-05).
#[derive(Debug, Serialize, Deserialize)]
pub struct JournalSummary {
    pub interstellar_count: i32,
    pub terrain_count: i32,
    pub missions: Vec<JournalEntry>,
}

/// Full detail returned by get_mission_detail (document + team + reports + completion status).
#[derive(Debug, Serialize, Deserialize)]
pub struct MissionDetailBundle {
    pub mission: MissionDocument,
    pub team: Vec<TeamMemberCounter>,
    pub status_reports: Vec<StatusReportItem>,
    pub completion_request: Option<CompletionRequestItem>,
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AS-01: Get My Missions (mission inbox)
// ════════════════════════════════════════════════════════════════════════════════

/// Returns all missions assigned to the current astronaut.
/// Sorted newest first. Cached in Redis for 30m, invalidated on assignment or status change.
///
/// **Access:** Astronaut (or Administrator).
#[tauri::command]
pub async fn ast_get_my_missions(
    state: State<'_, AppState>,
) -> Result<Vec<MissionSummary>, AppError> {
    let user = crate::require_auth_any!(state, [Role::Astronaut, Role::TheWanderer]);

    let cache_key = format!("missions:active:{}", user.id);

    // Try Redis cache first
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<Vec<MissionSummary>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let missions = sqlx::query_as::<_, MissionSummary>(
        r#"
        SELECT m.id, m.title, m.type, m.danger_level, m.location, m.status,
               u.full_name AS assigned_by_name, m.created_at
        FROM missions m
        JOIN mission_assignments ma ON ma.mission_id = m.id
        JOIN users u ON u.id = m.assigned_by
        WHERE ma.astronaut_id = $1
          AND m.deleted_at IS NULL
        ORDER BY m.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    // Cache for 30 minutes
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&missions) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 1800).await;
        }
    }

    Ok(missions)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AS-02: Get Mission Detail (document + team counters + reports)
// ════════════════════════════════════════════════════════════════════════════════

/// Returns the full mission document, team member counters, status reports, and
/// any existing completion request for the given mission.
///
/// **Access:** Astronaut assigned to the mission, TheWanderer, or Administrator.
#[tauri::command]
pub async fn ast_get_mission_detail(
    state: State<'_, AppState>,
    mission_id: Uuid,
) -> Result<MissionDetailBundle, AppError> {
    let user = crate::require_auth_any!(state, [Role::Astronaut, Role::TheWanderer]);

    // Verify the astronaut is assigned to this mission (or is Wanderer/Admin)
    if user.role == Role::Astronaut {
        let assigned: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT mission_id FROM mission_assignments
            WHERE mission_id = $1 AND astronaut_id = $2
            "#,
        )
        .bind(mission_id)
        .bind(user.id)
        .fetch_optional(&state.db_pool)
        .await?;

        if assigned.is_none() {
            return Err(AppError::Forbidden);
        }
    }

    // Fetch mission document
    let mission = sqlx::query_as::<_, MissionDocument>(
        r#"
        SELECT m.id, m.title, m.type, m.danger_level, m.location,
               m.mission_objective, m.procedures, m.known_dangers,
               m.status, m.assigned_by, u.full_name AS assigned_by_name,
               m.created_at
        FROM missions m
        JOIN users u ON u.id = m.assigned_by
        WHERE m.id = $1 AND m.deleted_at IS NULL
        "#,
    )
    .bind(mission_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Mission not found.".into()))?;

    // Fetch team members with counters (UC-AS-06)
    let team = sqlx::query_as::<_, TeamMemberCounter>(
        r#"
        SELECT ma.astronaut_id, u.full_name,
               COALESCE(mc.interstellar_count, 0) AS interstellar_count,
               COALESCE(mc.terrain_count, 0) AS terrain_count
        FROM mission_assignments ma
        JOIN users u ON u.id = ma.astronaut_id
        LEFT JOIN mission_counters mc ON mc.astronaut_id = ma.astronaut_id
        WHERE ma.mission_id = $1
        ORDER BY u.full_name
        "#,
    )
    .bind(mission_id)
    .fetch_all(&state.db_pool)
    .await?;

    // Fetch status reports for this mission
    let status_reports = sqlx::query_as::<_, StatusReportItem>(
        r#"
        SELECT sr.id, sr.mission_id, sr.submitted_by,
               u.full_name AS submitter_name,
               sr.report_date, sr.month_tracker, sr.rag_status,
               sr.current_status, sr.issues_blockers,
               sr.collected_samples_last_month, sr.progress_last_month,
               sr.plans_next_month, sr.created_at
        FROM mission_status_reports sr
        JOIN users u ON u.id = sr.submitted_by
        WHERE sr.mission_id = $1 AND sr.deleted_at IS NULL
        ORDER BY sr.report_date DESC
        "#,
    )
    .bind(mission_id)
    .fetch_all(&state.db_pool)
    .await?;

    // Fetch latest completion request (if any)
    let completion_request = sqlx::query_as::<_, CompletionRequestItem>(
        r#"
        SELECT id, mission_id, submitted_by, findings_summary,
               evidence_storage_paths, status, wanderer_note, taskmaster_note,
               created_at
        FROM mission_completion_requests
        WHERE mission_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(mission_id)
    .fetch_optional(&state.db_pool)
    .await?;

    Ok(MissionDetailBundle {
        mission,
        team,
        status_reports,
        completion_request,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AS-03: Submit Mission Status Report
// ════════════════════════════════════════════════════════════════════════════════

/// Submit a periodic status report for an active mission.
/// Delivered to The Wanderer in real time via notifications.
///
/// **Access:** Astronaut assigned to the mission (or Administrator).
#[tauri::command]
pub async fn ast_submit_status_report(
    state: State<'_, AppState>,
    payload: SubmitStatusReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth!(state, Role::Astronaut);

    // Validate RAG status if provided
    if let Some(ref rag) = payload.rag_status {
        if !["red", "amber", "green"].contains(&rag.as_str()) {
            return Err(AppError::Internal("Invalid RAG status. Must be red, amber, or green.".into()));
        }
    }

    if payload.current_status.trim().is_empty() {
        return Err(AppError::Internal("Current status cannot be empty.".into()));
    }

    // Verify astronaut is assigned to this mission
    let assigned: Option<(Uuid,)> = sqlx::query_as(
        "SELECT mission_id FROM mission_assignments WHERE mission_id = $1 AND astronaut_id = $2",
    )
    .bind(payload.mission_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    if assigned.is_none() {
        return Err(AppError::Internal("You are not assigned to this mission.".into()));
    }

    // Verify mission is active
    let mission_status: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM missions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.mission_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (status,) = mission_status
        .ok_or_else(|| AppError::Internal("Mission not found.".into()))?;

    if status != "active" && status != "completion_requested" {
        return Err(AppError::Internal("Can only submit reports for active missions.".into()));
    }

    // Parse report_date — accept both "YYYY-MM-DD" (from <input type="date">) and full ISO-8601
    let report_date: DateTime<Utc> = if let Ok(dt) = payload.report_date.parse::<DateTime<Utc>>() {
        dt
    } else if let Ok(nd) = chrono::NaiveDate::parse_from_str(&payload.report_date, "%Y-%m-%d") {
        nd.and_hms_opt(0, 0, 0).unwrap().and_utc()
    } else {
        return Err(AppError::Internal("Invalid report_date format. Use YYYY-MM-DD or ISO-8601.".into()));
    };

    let report_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO mission_status_reports
          (mission_id, submitted_by, report_date, month_tracker, rag_status,
           current_status, issues_blockers, collected_samples_last_month,
           progress_last_month, plans_next_month)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
    )
    .bind(payload.mission_id)
    .bind(user.id)
    .bind(report_date)
    .bind(&payload.month_tracker)
    .bind(&payload.rag_status)
    .bind(&payload.current_status)
    .bind(&payload.issues_blockers)
    .bind(&payload.collected_samples_last_month)
    .bind(&payload.progress_last_month)
    .bind(&payload.plans_next_month)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify The Wanderer (find the Wanderer user)
    let wanderer: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'TheWanderer' AND u.deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((wanderer_id,)) = wanderer {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'mission:status_report', $2::jsonb)
            "#,
        )
        .bind(wanderer_id)
        .bind(serde_json::json!({
            "report_id": report_id.0,
            "mission_id": payload.mission_id,
            "astronaut": user.full_name,
            "rag_status": payload.rag_status,
            "month_tracker": payload.month_tracker,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "mission_status_reports",
        report_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "mission_id": payload.mission_id,
            "rag_status": payload.rag_status,
            "current_status": payload.current_status,
        })),
    )
    .await?;

    Ok(report_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AS-04: Submit Mission Completion Request (with evidence file upload)
// ════════════════════════════════════════════════════════════════════════════════

/// Submit a mission completion request with evidence files uploaded to Supabase Storage.
/// Routes to The Wanderer for initial review.
///
/// Evidence files are uploaded first; if any upload fails the request is NOT created.
///
/// **Access:** Astronaut assigned to the mission (or Administrator).
#[tauri::command]
pub async fn ast_submit_completion_request(
    state: State<'_, AppState>,
    payload: SubmitCompletionRequestPayload,
    file_bytes_list: Vec<Vec<u8>>,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth!(state, Role::Astronaut);

    if payload.findings_summary.trim().is_empty() {
        return Err(AppError::Internal("Findings summary cannot be empty.".into()));
    }

    if payload.file_names.len() != file_bytes_list.len()
        || payload.file_names.len() != payload.content_types.len()
    {
        return Err(AppError::Internal("File names, content types, and file bytes must have matching lengths.".into()));
    }

    // Verify assignment
    let assigned: Option<(Uuid,)> = sqlx::query_as(
        "SELECT mission_id FROM mission_assignments WHERE mission_id = $1 AND astronaut_id = $2",
    )
    .bind(payload.mission_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    if assigned.is_none() {
        return Err(AppError::Internal("You are not assigned to this mission.".into()));
    }

    // Verify mission is active
    let mission_status: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM missions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.mission_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (status,) = mission_status
        .ok_or_else(|| AppError::Internal("Mission not found.".into()))?;

    if status != "active" {
        return Err(AppError::Internal("Can only submit completion for active missions.".into()));
    }

    // Check no pending completion request already exists
    let existing: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM mission_completion_requests
        WHERE mission_id = $1 AND status IN ('pending_wanderer', 'pending_taskmaster')
          AND deleted_at IS NULL
        "#,
    )
    .bind(payload.mission_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Internal("A completion request is already pending for this mission.".into()));
    }

    // Generate a request ID upfront so Storage paths match the DB record
    let request_id = Uuid::new_v4();

    // Upload evidence files to Supabase Storage
    let client = reqwest::Client::new();

    // Ensure bucket exists (idempotent)
    let _ = client
        .post(&format!("{}/bucket", state.supabase_storage_url))
        .header("Authorization", format!("Bearer {}", state.supabase_service_jwt))
        .header("Content-Type", "application/json")
        .body(r#"{"id":"rusa-files","name":"rusa-files","public":false}"#)
        .send()
        .await;

    let mut storage_paths: Vec<String> = Vec::new();

    for (i, file_bytes) in file_bytes_list.into_iter().enumerate() {
        let filename = &payload.file_names[i];
        let content_type = &payload.content_types[i];
        let file_uuid = Uuid::new_v4();
        let storage_path = format!(
            "missions/completion-evidence/{}/{}-{}",
            request_id, file_uuid, filename
        );

        let upload_url = format!(
            "{}/object/rusa-files/{}",
            state.supabase_storage_url, storage_path
        );

        let resp = client
            .post(&upload_url)
            .header("Authorization", format!("Bearer {}", state.supabase_service_jwt))
            .header("Content-Type", content_type)
            .body(file_bytes)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Evidence upload failed: {}", e)))?;

        if !resp.status().is_success() {
            let status_code = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Evidence upload returned {}: {}",
                status_code, body
            )));
        }

        storage_paths.push(storage_path);
    }

    // Insert completion request with the collected storage paths
    sqlx::query(
        r#"
        INSERT INTO mission_completion_requests
          (id, mission_id, submitted_by, findings_summary, evidence_storage_paths, status)
        VALUES ($1, $2, $3, $4, $5, 'pending_wanderer')
        "#,
    )
    .bind(request_id)
    .bind(payload.mission_id)
    .bind(user.id)
    .bind(&payload.findings_summary)
    .bind(&storage_paths)
    .fetch_optional(&state.db_pool)
    .await?;

    // Update mission status to completion_requested
    sqlx::query(
        "UPDATE missions SET status = 'completion_requested', updated_at = NOW() WHERE id = $1",
    )
    .bind(payload.mission_id)
    .execute(&state.db_pool)
    .await?;

    // Notify The Wanderer
    let wanderer: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'TheWanderer' AND u.deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((wanderer_id,)) = wanderer {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'mission:completion_submitted', $2::jsonb)
            "#,
        )
        .bind(wanderer_id)
        .bind(serde_json::json!({
            "request_id": request_id,
            "mission_id": payload.mission_id,
            "astronaut": user.full_name,
            "findings_summary": payload.findings_summary,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    // Invalidate mission cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("missions:active:{}", user.id)).await;
    }

    write_audit_log(
        &state.db_pool,
        "mission_completion_requests",
        request_id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "mission_id": payload.mission_id,
            "findings_summary": payload.findings_summary,
            "evidence_count": storage_paths.len(),
        })),
    )
    .await?;

    Ok(request_id)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AS-05: Personal Journal / Mission Counter
// ════════════════════════════════════════════════════════════════════════════════

/// Returns the astronaut's completed missions list and official interstellar/terrain counters.
/// Cached in Redis for 1h, invalidated on counter increment.
///
/// **Access:** Astronaut (or Administrator).
#[tauri::command]
pub async fn ast_get_personal_journal(
    state: State<'_, AppState>,
) -> Result<JournalSummary, AppError> {
    let user = crate::require_auth_any!(state, [Role::Astronaut, Role::TheWanderer]);

    let counter_cache_key = format!("mission_counter:{}", user.id);

    // Try Redis cache for counters
    let mut cached_counters: Option<(i32, i32)> = None;
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&counter_cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<(i32, i32)>(&cached) {
                cached_counters = Some(parsed);
            }
        }
    }

    let (interstellar_count, terrain_count) = if let Some(c) = cached_counters {
        c
    } else {
        let row: Option<(i32, i32)> = sqlx::query_as(
            "SELECT interstellar_count, terrain_count FROM mission_counters WHERE astronaut_id = $1",
        )
        .bind(user.id)
        .fetch_optional(&state.db_pool)
        .await?;

        let counts = row.unwrap_or((0, 0));

        // Cache counters for 1h
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(json) = serde_json::to_string(&counts) {
                let _: Result<(), _> = conn.set_ex(&counter_cache_key, &json, 3600).await;
            }
        }

        counts
    };

    // Fetch completed missions
    let missions = sqlx::query_as::<_, JournalEntry>(
        r#"
        SELECT m.id AS mission_id, m.title, m.type, m.location, m.danger_level,
               m.updated_at AS completed_at
        FROM missions m
        JOIN mission_assignments ma ON ma.mission_id = m.id
        WHERE ma.astronaut_id = $1
          AND m.status = 'completed'
          AND m.deleted_at IS NULL
        ORDER BY m.updated_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(JournalSummary {
        interstellar_count,
        terrain_count,
        missions,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AS-06: View Colleague Mission Counters
// ════════════════════════════════════════════════════════════════════════════════

/// Get mission counters for all astronauts assigned to a specific mission.
/// Used in the team panel on the mission detail view.
///
/// **Access:** Astronaut assigned to the mission, TheWanderer, or Administrator.
#[tauri::command]
pub async fn ast_get_colleague_counters(
    state: State<'_, AppState>,
    mission_id: Uuid,
) -> Result<Vec<TeamMemberCounter>, AppError> {
    let user = crate::require_auth_any!(state, [Role::Astronaut, Role::TheWanderer]);

    // Verify the astronaut is assigned (Wanderer/Admin bypasses)
    if user.role == Role::Astronaut {
        let assigned: Option<(Uuid,)> = sqlx::query_as(
            "SELECT mission_id FROM mission_assignments WHERE mission_id = $1 AND astronaut_id = $2",
        )
        .bind(mission_id)
        .bind(user.id)
        .fetch_optional(&state.db_pool)
        .await?;

        if assigned.is_none() {
            return Err(AppError::Forbidden);
        }
    }

    let team = sqlx::query_as::<_, TeamMemberCounter>(
        r#"
        SELECT ma.astronaut_id, u.full_name,
               COALESCE(mc.interstellar_count, 0) AS interstellar_count,
               COALESCE(mc.terrain_count, 0) AS terrain_count
        FROM mission_assignments ma
        JOIN users u ON u.id = ma.astronaut_id
        LEFT JOIN mission_counters mc ON mc.astronaut_id = ma.astronaut_id
        WHERE ma.mission_id = $1
        ORDER BY u.full_name
        "#,
    )
    .bind(mission_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(team)
}

// ════════════════════════════════════════════════════════════════════════════════
// Director-side: Wanderer — Assign Mission (UC-WAN-01)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct AssignMissionPayload {
    pub title: String,
    #[serde(rename = "type")]
    pub mission_type: String,
    pub danger_level: Option<String>,
    pub location: String,
    pub mission_objective: Option<String>,
    pub procedures: Option<String>,
    pub known_dangers: Option<String>,
    pub astronaut_ids: Vec<Uuid>,
}

/// Create a new mission and assign astronauts (UC-WAN-01).
/// Sends real-time notifications to all assigned astronauts.
///
/// **Access:** TheWanderer, TheTaskmaster, or Administrator.
#[tauri::command]
pub async fn ast_assign_mission(
    state: State<'_, AppState>,
    payload: AssignMissionPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::TheWanderer, Role::TheTaskmaster]);

    if payload.title.trim().is_empty() {
        return Err(AppError::Internal("Mission title cannot be empty.".into()));
    }

    if !["interstellar", "terrain"].contains(&payload.mission_type.as_str()) {
        return Err(AppError::Internal("Mission type must be 'interstellar' or 'terrain'.".into()));
    }

    if let Some(ref dl) = payload.danger_level {
        if !["low", "medium", "high", "critical"].contains(&dl.as_str()) {
            return Err(AppError::Internal("Invalid danger level.".into()));
        }
    }

    if payload.astronaut_ids.is_empty() {
        return Err(AppError::Internal("At least one astronaut must be assigned.".into()));
    }

    if payload.location.trim().is_empty() {
        return Err(AppError::Internal("Mission location cannot be empty.".into()));
    }

    // Validate all target users are Astronauts
    for &aid in &payload.astronaut_ids {
        let role_check: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT r.name FROM users u JOIN roles r ON r.id = u.role_id
            WHERE u.id = $1 AND u.deleted_at IS NULL
            "#,
        )
        .bind(aid)
        .fetch_optional(&state.db_pool)
        .await?;

        match role_check {
            Some((name,)) if name == "Astronaut" => {}
            _ => {
                return Err(AppError::Internal(format!(
                    "User {} is not an active Astronaut.",
                    aid
                )));
            }
        }
    }

    // Create mission
    let mission_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO missions
          (title, type, danger_level, location, mission_objective, procedures, known_dangers, assigned_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.mission_type)
    .bind(&payload.danger_level)
    .bind(&payload.location)
    .bind(&payload.mission_objective)
    .bind(&payload.procedures)
    .bind(&payload.known_dangers)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Assign astronauts + notify
    for &aid in &payload.astronaut_ids {
        sqlx::query(
            "INSERT INTO mission_assignments (mission_id, astronaut_id) VALUES ($1, $2)",
        )
        .bind(mission_id.0)
        .bind(aid)
        .execute(&state.db_pool)
        .await?;

        // Notify astronaut
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'mission:assigned', $2::jsonb)
            "#,
        )
        .bind(aid)
        .bind(serde_json::json!({
            "mission_id": mission_id.0,
            "title": payload.title,
            "type": payload.mission_type,
            "danger_level": payload.danger_level,
            "location": payload.location,
            "assigned_by": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;

        // Initialize counter row if not exists
        sqlx::query(
            r#"
            INSERT INTO mission_counters (astronaut_id)
            VALUES ($1)
            ON CONFLICT (astronaut_id) DO NOTHING
            "#,
        )
        .bind(aid)
        .execute(&state.db_pool)
        .await?;

        // Invalidate mission cache for astronaut
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.del(format!("missions:active:{}", aid)).await;
        }
    }

    write_audit_log(
        &state.db_pool,
        "missions",
        mission_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "title": payload.title,
            "type": payload.mission_type,
            "astronaut_ids": payload.astronaut_ids,
        })),
    )
    .await?;

    Ok(mission_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// Director-side: Wanderer — Process Completion Request (UC-WAN-02)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ProcessCompletionPayload {
    pub request_id: Uuid,
    pub decision: String,   // 'forward' (to Taskmaster) or 'reject'
    pub note: Option<String>,
}

/// Wanderer processes a mission completion request.
/// If decision = 'forward', routes to Taskmaster. If 'reject', returns to astronaut.
///
/// **Access:** TheWanderer or Administrator.
#[tauri::command]
pub async fn ast_process_completion_request(
    state: State<'_, AppState>,
    payload: ProcessCompletionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::TheWanderer, Role::Administrator]);

    if payload.decision != "forward" && payload.decision != "reject" {
        return Err(AppError::Internal("Decision must be 'forward' or 'reject'.".into()));
    }

    // Fetch existing request
    let existing: Option<(Uuid, String, Uuid)> = sqlx::query_as(
        r#"
        SELECT id, status, submitted_by FROM mission_completion_requests
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (_, old_status, submitter_id) = existing
        .ok_or_else(|| AppError::Internal("Completion request not found.".into()))?;

    if old_status != "pending_wanderer" {
        return Err(AppError::Internal("This request is not pending Wanderer review.".into()));
    }

    let new_status = if payload.decision == "forward" {
        "pending_taskmaster"
    } else {
        "rejected"
    };

    sqlx::query(
        r#"
        UPDATE mission_completion_requests
        SET status = $2, wanderer_note = $3, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(payload.request_id)
    .bind(new_status)
    .bind(&payload.note)
    .execute(&state.db_pool)
    .await?;

    // If rejected, revert mission to active
    if payload.decision == "reject" {
        let mission_id: Option<(Uuid,)> = sqlx::query_as(
            "SELECT mission_id FROM mission_completion_requests WHERE id = $1",
        )
        .bind(payload.request_id)
        .fetch_optional(&state.db_pool)
        .await?;

        if let Some((mid,)) = mission_id {
            sqlx::query("UPDATE missions SET status = 'active', updated_at = NOW() WHERE id = $1")
                .bind(mid)
                .execute(&state.db_pool)
                .await?;
        }
    }

    // Notify: if forwarded → Taskmaster; if rejected → astronaut
    if payload.decision == "forward" {
        let taskmaster: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id
            WHERE r.name = 'TheTaskmaster' AND u.deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .fetch_optional(&state.db_pool)
        .await?;

        if let Some((tm_id,)) = taskmaster {
            sqlx::query(
                r#"
                INSERT INTO notifications (user_id, type, payload)
                VALUES ($1, 'mission:completion:pending_taskmaster', $2::jsonb)
                "#,
            )
            .bind(tm_id)
            .bind(serde_json::json!({
                "request_id": payload.request_id,
                "wanderer_note": payload.note,
            }))
            .execute(&state.db_pool)
            .await?;
        }

        // Also notify astronaut that it's been forwarded
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'mission:completion:wanderer_processed', $2::jsonb)
            "#,
        )
        .bind(submitter_id)
        .bind(serde_json::json!({
            "request_id": payload.request_id,
            "status": "forwarded_to_taskmaster",
        }))
        .execute(&state.db_pool)
        .await?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'mission:completion:rejected', $2::jsonb)
            "#,
        )
        .bind(submitter_id)
        .bind(serde_json::json!({
            "request_id": payload.request_id,
            "reason": payload.note,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "mission_completion_requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": old_status })),
        Some(serde_json::json!({
            "status": new_status,
            "wanderer_note": payload.note,
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// Director-side: Taskmaster — Approve Mission Completion (UC-TM-03 variant)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct TaskmasterMissionDecisionPayload {
    pub request_id: Uuid,
    pub decision: String,   // 'approved' or 'rejected'
    pub note: Option<String>,
}

/// Taskmaster approves or rejects a mission completion forwarded by the Wanderer.
/// On approval: mission status → completed, counters incremented, Redis invalidated.
///
/// **Access:** TheTaskmaster or Administrator.
#[tauri::command]
pub async fn ast_taskmaster_decide_completion(
    state: State<'_, AppState>,
    payload: TaskmasterMissionDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::TheTaskmaster, Role::Administrator]);

    if payload.decision != "approved" && payload.decision != "rejected" {
        return Err(AppError::Internal("Decision must be 'approved' or 'rejected'.".into()));
    }

    // Fetch request
    let existing: Option<(Uuid, String, Uuid, Uuid)> = sqlx::query_as(
        r#"
        SELECT id, status, submitted_by, mission_id
        FROM mission_completion_requests
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (_, old_status, submitter_id, mission_id) = existing
        .ok_or_else(|| AppError::Internal("Completion request not found.".into()))?;

    if old_status != "pending_taskmaster" {
        return Err(AppError::Internal("This request is not pending Taskmaster review.".into()));
    }

    let new_status = if payload.decision == "approved" {
        "approved"
    } else {
        "rejected"
    };

    sqlx::query(
        r#"
        UPDATE mission_completion_requests
        SET status = $2, taskmaster_note = $3, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(payload.request_id)
    .bind(new_status)
    .bind(&payload.note)
    .execute(&state.db_pool)
    .await?;

    if payload.decision == "approved" {
        // Update mission status to completed
        sqlx::query(
            "UPDATE missions SET status = 'completed', updated_at = NOW() WHERE id = $1",
        )
        .bind(mission_id)
        .execute(&state.db_pool)
        .await?;

        // Get mission type for counter increment
        let mission_type: Option<(String,)> = sqlx::query_as(
            "SELECT type FROM missions WHERE id = $1",
        )
        .bind(mission_id)
        .fetch_optional(&state.db_pool)
        .await?;

        if let Some((mtype,)) = mission_type {
            // Increment counters for ALL assigned astronauts
            let astronauts: Vec<(Uuid,)> = sqlx::query_as(
                "SELECT astronaut_id FROM mission_assignments WHERE mission_id = $1",
            )
            .bind(mission_id)
            .fetch_all(&state.db_pool)
            .await?;

            for (aid,) in &astronauts {
                let column = if mtype == "interstellar" {
                    "interstellar_count"
                } else {
                    "terrain_count"
                };

                sqlx::query(&format!(
                    r#"
                    INSERT INTO mission_counters (astronaut_id, {col})
                    VALUES ($1, 1)
                    ON CONFLICT (astronaut_id)
                    DO UPDATE SET {col} = mission_counters.{col} + 1, updated_at = NOW()
                    "#,
                    col = column
                ))
                .bind(aid)
                .execute(&state.db_pool)
                .await?;

                // Invalidate Redis caches
                if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
                    let _: Result<(), _> = conn.del(format!("mission_counter:{}", aid)).await;
                    let _: Result<(), _> = conn.del(format!("missions:active:{}", aid)).await;
                }
            }
        }

        // Notify astronaut of approval
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'mission:completion:approved', $2::jsonb)
            "#,
        )
        .bind(submitter_id)
        .bind(serde_json::json!({
            "request_id": payload.request_id,
            "mission_id": mission_id,
            "note": payload.note,
        }))
        .execute(&state.db_pool)
        .await?;
    } else {
        // Rejected — revert mission to active
        sqlx::query("UPDATE missions SET status = 'active', updated_at = NOW() WHERE id = $1")
            .bind(mission_id)
            .execute(&state.db_pool)
            .await?;

        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'mission:completion:rejected', $2::jsonb)
            "#,
        )
        .bind(submitter_id)
        .bind(serde_json::json!({
            "request_id": payload.request_id,
            "reason": payload.note,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "mission_completion_requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": old_status })),
        Some(serde_json::json!({
            "status": new_status,
            "taskmaster_note": payload.note,
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// Director-side: Wanderer — Get All Missions overview
// ════════════════════════════════════════════════════════════════════════════════

/// Returns every mission (newest first) for the Wanderer / Taskmaster overview.
///
/// **Access:** TheWanderer, TheTaskmaster, or Administrator.
#[tauri::command]
pub async fn ast_get_all_missions(
    state: State<'_, AppState>,
) -> Result<Vec<MissionSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::TheWanderer, Role::TheTaskmaster, Role::Administrator
    ]);

    let missions = sqlx::query_as::<_, MissionSummary>(
        r#"
        SELECT m.id, m.title, m.type, m.danger_level, m.location, m.status,
               u.full_name AS assigned_by_name, m.created_at
        FROM missions m
        JOIN users u ON u.id = m.assigned_by
        WHERE m.deleted_at IS NULL
        ORDER BY m.created_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(missions)
}

// ════════════════════════════════════════════════════════════════════════════════
// Director-side: Wanderer — Get Mission Status Reports (UC-WAN-03)
// ════════════════════════════════════════════════════════════════════════════════

/// Get all mission status reports for the Wanderer's overview feed.
/// Organized by mission, sorted by most recent first.
///
/// **Access:** TheWanderer, TheTaskmaster, or Administrator.
#[tauri::command]
pub async fn ast_get_all_status_reports(
    state: State<'_, AppState>,
) -> Result<Vec<StatusReportItem>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::TheWanderer, Role::TheTaskmaster, Role::Administrator
    ]);

    let reports = sqlx::query_as::<_, StatusReportItem>(
        r#"
        SELECT sr.id, sr.mission_id, sr.submitted_by,
               u.full_name AS submitter_name,
               sr.report_date, sr.month_tracker, sr.rag_status,
               sr.current_status, sr.issues_blockers,
               sr.collected_samples_last_month, sr.progress_last_month,
               sr.plans_next_month, sr.created_at
        FROM mission_status_reports sr
        JOIN users u ON u.id = sr.submitted_by
        WHERE sr.deleted_at IS NULL
        ORDER BY sr.report_date DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(reports)
}

// ════════════════════════════════════════════════════════════════════════════════
// Director-side: Wanderer — Get Completion Requests Queue (UC-WAN-02)
// ════════════════════════════════════════════════════════════════════════════════

/// Completion request summary with mission title for the Wanderer/Taskmaster queue.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CompletionRequestSummary {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub mission_title: String,
    pub submitted_by: Uuid,
    pub submitter_name: String,
    pub findings_summary: String,
    pub evidence_storage_paths: Option<Vec<String>>,
    pub status: String,
    pub wanderer_note: Option<String>,
    pub taskmaster_note: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Get pending completion requests for the Wanderer's review queue.
///
/// **Access:** TheWanderer or Administrator.
#[tauri::command]
pub async fn ast_get_completion_requests_wanderer(
    state: State<'_, AppState>,
) -> Result<Vec<CompletionRequestSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::TheWanderer, Role::Administrator]);

    let requests = sqlx::query_as::<_, CompletionRequestSummary>(
        r#"
        SELECT cr.id, cr.mission_id, m.title AS mission_title,
               cr.submitted_by, u.full_name AS submitter_name,
               cr.findings_summary, cr.evidence_storage_paths,
               cr.status, cr.wanderer_note, cr.taskmaster_note,
               cr.created_at
        FROM mission_completion_requests cr
        JOIN missions m ON m.id = cr.mission_id
        JOIN users u ON u.id = cr.submitted_by
        WHERE cr.deleted_at IS NULL
        ORDER BY cr.created_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(requests)
}

/// Get pending completion requests for the Taskmaster's queue (forwarded by Wanderer).
///
/// **Access:** TheTaskmaster or Administrator.
#[tauri::command]
pub async fn ast_get_completion_requests_taskmaster(
    state: State<'_, AppState>,
) -> Result<Vec<CompletionRequestSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::TheTaskmaster, Role::Administrator]);

    let requests = sqlx::query_as::<_, CompletionRequestSummary>(
        r#"
        SELECT cr.id, cr.mission_id, m.title AS mission_title,
               cr.submitted_by, u.full_name AS submitter_name,
               cr.findings_summary, cr.evidence_storage_paths,
               cr.status, cr.wanderer_note, cr.taskmaster_note,
               cr.created_at
        FROM mission_completion_requests cr
        JOIN missions m ON m.id = cr.mission_id
        JOIN users u ON u.id = cr.submitted_by
        WHERE cr.status = 'pending_taskmaster'
          AND cr.deleted_at IS NULL
        ORDER BY cr.created_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(requests)
}

// ════════════════════════════════════════════════════════════════════════════════
// Signed URL generation for evidence files
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct EvidenceFileUrl {
    pub storage_path: String,
    pub signed_url: String,
    pub filename: String,
}

/// Generate signed URLs for evidence files attached to a completion request.
///
/// **Access:** Astronaut assigned to the mission, TheWanderer, TheTaskmaster, or Administrator.
#[tauri::command]
pub async fn ast_get_evidence_urls(
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<Vec<EvidenceFileUrl>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Astronaut, Role::TheWanderer, Role::TheTaskmaster
    ]);

    // Fetch the completion request
    let req: Option<(Uuid, Option<Vec<String>>)> = sqlx::query_as(
        r#"
        SELECT mission_id, evidence_storage_paths
        FROM mission_completion_requests
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (mission_id, paths_opt) = req
        .ok_or_else(|| AppError::Internal("Completion request not found.".into()))?;

    // If astronaut, verify assignment
    if user.role == Role::Astronaut {
        let assigned: Option<(Uuid,)> = sqlx::query_as(
            "SELECT mission_id FROM mission_assignments WHERE mission_id = $1 AND astronaut_id = $2",
        )
        .bind(mission_id)
        .bind(user.id)
        .fetch_optional(&state.db_pool)
        .await?;

        if assigned.is_none() {
            return Err(AppError::Forbidden);
        }
    }

    let paths = paths_opt.unwrap_or_default();
    let client = reqwest::Client::new();
    let mut result = Vec::new();

    for path in paths {
        // Extract filename from storage path (last segment after the UUID prefix)
        let filename = path
            .rsplit('/')
            .next()
            .and_then(|s| s.split_once('-').map(|(_, name)| name.to_string()))
            .unwrap_or_else(|| path.clone());

        // Generate signed URL via Supabase Storage API
        let sign_url = format!(
            "{}/object/sign/rusa-files/{}",
            state.supabase_storage_url, path
        );

        let resp = client
            .post(&sign_url)
            .header("Authorization", format!("Bearer {}", state.supabase_service_jwt))
            .header("Content-Type", "application/json")
            .body(r#"{"expiresIn":3600}"#)
            .send()
            .await;

        let signed_url = match resp {
            Ok(r) if r.status().is_success() => {
                let text = r.text().await.unwrap_or_default();
                let body: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                let token = body["signedURL"].as_str().unwrap_or("");
                if token.starts_with("http") {
                    token.to_string()
                } else {
                    format!("{}{}", state.supabase_storage_url, token)
                }
            }
            _ => String::new(), // fallback: empty URL if signing fails
        };

        result.push(EvidenceFileUrl {
            storage_path: path,
            signed_url,
            filename,
        });
    }

    Ok(result)
}

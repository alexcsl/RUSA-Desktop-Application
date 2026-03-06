// commands/space_station.rs — Space Station Settlers subsystem commands
// Source of truth: 08_SPACE_STATION_SETTLERS.md, 00_MASTER_GUIDE.md
//
// Use cases implemented:
//   Space Station Settler:
//     UC-SSS-01: Report Finding to Galactic Security    (sst_report_finding_to_security)
//     UC-SSS-02: Add Finding to Private Station Archive (sst_add_to_archive)
//     UC-SSS-08: View Private Station Archive           (sst_get_archive)
//     UC-SSS-03: Manage Station Inventory               (sst_manage_inventory + sst_get_inventory)
//     UC-SSS-04: Submit Supply Request                  (sst_submit_supply_request + sst_get_supply_requests)
//     UC-SSS-05: Upload and Annotate Station Map        (sst_upload_map + sst_publish_map + sst_add_annotation + sst_delete_annotation)
//     UC-SSS-06: Log Personnel On Board                 (sst_log_arrival + sst_log_departure + sst_get_personnel)
//     UC-SSS-07: Propose Station Abandonment            (sst_propose_abandonment)
//
//   Visitor (no auth required):
//     UC-SSV-01: View Interactive Station Map           (sst_get_published_map)

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  STRUCTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ── Station lookup ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, FromRow)]
pub struct StationSummary {
    pub id: Uuid,
    pub name: String,
    pub sector: String,
    pub status: String,
    pub nearest_galactic_security_team_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// ── Inventory ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ManageInventoryPayload {
    pub station_id: Uuid,
    pub item_name: String,
    pub category: Option<String>,
    pub quantity_change: i32, // positive = add, negative = remove
    pub unit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub station_id: Uuid,
    pub item_name: String,
    pub category: Option<String>,
    pub quantity: i32,
    pub unit: Option<String>,
    pub updated_at: DateTime<Utc>,
}

// ── Archive ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddToArchivePayload {
    pub station_id: Uuid,
    pub finding_type: String,
    pub archive_data: serde_json::Value,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ArchiveEntry {
    pub id: Uuid,
    pub station_id: Uuid,
    pub logged_by: Uuid,
    pub logger_name: String,
    pub finding_type: Option<String>,
    pub archive_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ── Supply Request ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SubmitSupplyRequestPayload {
    pub station_id: Uuid,
    pub items: serde_json::Value, // [{item, quantity, justification}]
}

#[derive(Debug, Serialize, FromRow)]
pub struct SupplyRequestSummary {
    pub id: Uuid,
    pub station_id: Uuid,
    pub submitted_by: Uuid,
    pub submitter_name: String,
    pub items: serde_json::Value,
    pub status: String,
    pub vote_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// ── Map ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, FromRow)]
pub struct StationMapSummary {
    pub id: Uuid,
    pub station_id: Uuid,
    pub image_storage_path: String,
    pub image_width: Option<f64>,
    pub image_height: Option<f64>,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddAnnotationPayload {
    pub map_id: Uuid,
    pub label: String,
    pub description: Option<String>,
    pub x_position: f64,
    pub y_position: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MapAnnotation {
    pub id: Uuid,
    pub map_id: Uuid,
    pub label: String,
    pub description: Option<String>,
    pub x_position: f64,
    pub y_position: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishedMapData {
    pub signed_url: String,
    pub image_width: Option<f64>,
    pub image_height: Option<f64>,
    pub annotations: Vec<MapAnnotation>,
    pub station_name: String,
}

// ── Personnel Log ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LogArrivalPayload {
    pub station_id: Uuid,
    pub person_name: String,
    pub role: Option<String>,
    pub arrived_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PersonnelLogEntry {
    pub id: Uuid,
    pub station_id: Uuid,
    pub person_name: String,
    pub role: Option<String>,
    pub arrived_at: DateTime<Utc>,
    pub departed_at: Option<DateTime<Utc>>,
    pub logged_by: Uuid,
    pub created_at: DateTime<Utc>,
}

// ── Finding Report ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ReportFindingPayload {
    pub station_id: Uuid,
    pub incident_type: String,
    pub location: String,
    pub occurred_at: Option<String>,
    pub description: String,
    pub severity: String,
    pub recommended_action: Option<String>,
    pub sector_or_base: Option<String>,
    pub incident_meta: Option<serde_json::Value>,
}

// ── Abandonment ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ProposeAbandonmentPayload {
    pub station_id: Uuid,
    pub finding_reference_id: Option<Uuid>,
    pub reason: String,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  HELPER — resolve settler's station
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Get the station this settler is assigned to (based on base_location sector match).
/// Falls back to first active station if no match found.
async fn get_settler_station(
    pool: &sqlx::PgPool,
) -> Result<StationSummary, AppError> {
    let station = sqlx::query_as::<_, StationSummary>(
        r#"
        SELECT id, name, sector, status, nearest_galactic_security_team_id, created_at
        FROM space_stations
        WHERE status = 'active' AND deleted_at IS NULL
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Internal("No active space station found.".into()))?;

    Ok(station)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  COMMANDS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSS-01: Report Finding to Galactic Security
// ════════════════════════════════════════════════════════════════════════════════

/// Settler reports a finding to the nearest Galactic Security team.
/// Creates an incident_report with source='external_report' and notifies the security team head.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_report_finding_to_security(
    state: State<'_, AppState>,
    payload: ReportFindingPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let occurred = payload
        .occurred_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    // Insert into incident_reports (shared with Security Teams)
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO incident_reports
          (reported_by, source, incident_type, location, occurred_at, description,
           severity, recommended_action, sector_or_base, incident_meta)
        VALUES ($1, 'external_report', $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.incident_type)
    .bind(&payload.location)
    .bind(occurred)
    .bind(&payload.description)
    .bind(&payload.severity)
    .bind(&payload.recommended_action)
    .bind(&payload.sector_or_base)
    .bind(&payload.incident_meta)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify nearest security head
    let station = get_settler_station(&state.db_pool).await?;
    if let Some(sec_head_id) = station.nearest_galactic_security_team_id {
        let _ = sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'report:received', $2)
            "#,
        )
        .bind(sec_head_id)
        .bind(serde_json::json!({
            "report_type": "security_finding",
            "incident_type": payload.incident_type,
            "severity": payload.severity,
            "reported_by": user.full_name,
            "station": station.name,
        }))
        .execute(&state.db_pool)
        .await;
    }

    write_audit_log(
        &state.db_pool,
        "incident_reports",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "source": "external_report",
            "incident_type": payload.incident_type,
            "severity": payload.severity,
            "station": station.name,
        })),
    )
    .await?;

    Ok(row.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSS-02: Add Finding to Private Station Archive
// ════════════════════════════════════════════════════════════════════════════════

/// Add a finding entry to the station's private archive (JSONB document).
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_add_to_archive(
    state: State<'_, AppState>,
    payload: AddToArchivePayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO station_private_archive
          (station_id, logged_by, finding_type, archive_data)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(payload.station_id)
    .bind(user.id)
    .bind(&payload.finding_type)
    .bind(&payload.archive_data)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "station_private_archive",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "station_id": payload.station_id,
            "finding_type": payload.finding_type,
        })),
    )
    .await?;

    Ok(row.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSS-08: View Private Station Archive
// ════════════════════════════════════════════════════════════════════════════════

/// Get all archive entries for a station. Supports optional search filter.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_get_archive(
    state: State<'_, AppState>,
    station_id: Uuid,
    search: Option<String>,
) -> Result<Vec<ArchiveEntry>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let entries = if let Some(ref q) = search {
        let pattern = format!("%{}%", q);
        sqlx::query_as::<_, ArchiveEntry>(
            r#"
            SELECT a.id, a.station_id, a.logged_by,
                   u.full_name AS logger_name,
                   a.finding_type, a.archive_data, a.created_at
            FROM station_private_archive a
            JOIN users u ON u.id = a.logged_by
            WHERE a.station_id = $1 AND a.deleted_at IS NULL
              AND (a.finding_type ILIKE $2
                   OR a.archive_data::TEXT ILIKE $2)
            ORDER BY a.created_at DESC
            "#,
        )
        .bind(station_id)
        .bind(&pattern)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, ArchiveEntry>(
            r#"
            SELECT a.id, a.station_id, a.logged_by,
                   u.full_name AS logger_name,
                   a.finding_type, a.archive_data, a.created_at
            FROM station_private_archive a
            JOIN users u ON u.id = a.logged_by
            WHERE a.station_id = $1 AND a.deleted_at IS NULL
            ORDER BY a.created_at DESC
            "#,
        )
        .bind(station_id)
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(entries)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSS-03: Manage Station Inventory
// ════════════════════════════════════════════════════════════════════════════════

/// Add or remove inventory items. If the item exists, update quantity.
/// If quantity_change is negative and would go below 0, error.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_manage_inventory(
    state: State<'_, AppState>,
    payload: ManageInventoryPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    // Check if item already exists in this station's inventory
    let existing: Option<(Uuid, i32)> = sqlx::query_as(
        r#"
        SELECT id, quantity FROM station_inventory
        WHERE station_id = $1 AND LOWER(item_name) = LOWER($2) AND deleted_at IS NULL
        "#,
    )
    .bind(payload.station_id)
    .bind(&payload.item_name)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((item_id, current_qty)) = existing {
        let new_qty = current_qty + payload.quantity_change;
        if new_qty < 0 {
            return Err(AppError::Internal(format!(
                "Cannot remove {} units — only {} available.",
                -payload.quantity_change, current_qty
            )));
        }

        sqlx::query(
            r#"
            UPDATE station_inventory
            SET quantity = $1, last_updated_by = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(new_qty)
        .bind(user.id)
        .bind(item_id)
        .execute(&state.db_pool)
        .await?;

        write_audit_log(
            &state.db_pool,
            "station_inventory",
            item_id,
            AuditOperation::Update,
            user.id,
            Some(serde_json::json!({"quantity": current_qty})),
            Some(serde_json::json!({"quantity": new_qty})),
        )
        .await?;
    } else {
        if payload.quantity_change < 0 {
            return Err(AppError::Internal(
                "Item does not exist — cannot remove.".into(),
            ));
        }

        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO station_inventory
              (station_id, item_name, category, quantity, unit, last_updated_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(payload.station_id)
        .bind(&payload.item_name)
        .bind(&payload.category)
        .bind(payload.quantity_change)
        .bind(&payload.unit)
        .bind(user.id)
        .fetch_one(&state.db_pool)
        .await?;

        write_audit_log(
            &state.db_pool,
            "station_inventory",
            row.0,
            AuditOperation::Create,
            user.id,
            None,
            Some(serde_json::json!({
                "item_name": payload.item_name,
                "quantity": payload.quantity_change,
            })),
        )
        .await?;
    }

    // Invalidate cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .del(format!("station_inventory:{}", payload.station_id))
            .await;
    }

    Ok(())
}

/// Get current inventory for a station.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_get_inventory(
    state: State<'_, AppState>,
    station_id: Uuid,
) -> Result<Vec<InventoryItem>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    // Check Redis cache (15 min TTL)
    let cache_key = format!("station_inventory:{}", station_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(items) = serde_json::from_str::<Vec<InventoryItem>>(&cached) {
                return Ok(items);
            }
        }
    }

    let items = sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT id, station_id, item_name, category, quantity, unit, updated_at
        FROM station_inventory
        WHERE station_id = $1 AND deleted_at IS NULL
        ORDER BY category NULLS LAST, item_name
        "#,
    )
    .bind(station_id)
    .fetch_all(&state.db_pool)
    .await?;

    // Populate cache — 15 minutes
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&items) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 900).await;
        }
    }

    Ok(items)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSS-04: Submit Supply Request
// ════════════════════════════════════════════════════════════════════════════════

/// Submit a supply request. Creates a vote session for Directors.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_submit_supply_request(
    state: State<'_, AppState>,
    payload: SubmitSupplyRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    // Create vote session for Directors
    let vote_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions
          (topic, context, status, opens_at, created_by,
           total_yay, total_nay, total_abstain,
           admin_overridden, admin_terminated)
        VALUES ($1, $2, 'open', NOW(), $3, 0, 0, 0, FALSE, FALSE)
        RETURNING id
        "#,
    )
    .bind(format!("Supply Request — Station"))
    .bind(format!(
        "Supply request from {} for station supplies.",
        user.full_name
    ))
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Insert supply request
    let req_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO station_supply_requests
          (station_id, submitted_by, items, status, vote_session_id)
        VALUES ($1, $2, $3, 'pending_approval', $4)
        RETURNING id
        "#,
    )
    .bind(payload.station_id)
    .bind(user.id)
    .bind(&payload.items)
    .bind(vote_row.0)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify all Directors about the new vote
    let directors: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name IN (
          'GeneralDirector','TheDirector','TheAccountant','TheLibrarian','TheNomad',
          'TheArtificer','TheObserver','TheWanderer','TheTaskmaster','TheGuardian',
          'TheStatistician','TheCoordinator','TheOverseer','TheAnchorman'
        ) AND u.deleted_at IS NULL AND u.is_active = true
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (dir_id,) in &directors {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'vote:new', $2)"#,
        )
        .bind(dir_id)
        .bind(serde_json::json!({
            "topic": "Supply Request — Station",
            "session_id": vote_row.0,
            "requester": user.full_name,
        }))
        .execute(&state.db_pool)
        .await;
    }

    write_audit_log(
        &state.db_pool,
        "station_supply_requests",
        req_row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "station_id": payload.station_id,
            "vote_session_id": vote_row.0,
        })),
    )
    .await?;

    Ok(req_row.0)
}

/// Get supply requests for a station.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_get_supply_requests(
    state: State<'_, AppState>,
    station_id: Uuid,
) -> Result<Vec<SupplyRequestSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let reqs = sqlx::query_as::<_, SupplyRequestSummary>(
        r#"
        SELECT r.id, r.station_id, r.submitted_by,
               u.full_name AS submitter_name,
               r.items, r.status, r.vote_session_id, r.created_at
        FROM station_supply_requests r
        JOIN users u ON u.id = r.submitted_by
        WHERE r.station_id = $1 AND r.deleted_at IS NULL
        ORDER BY r.created_at DESC
        "#,
    )
    .bind(station_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(reqs)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSS-05: Upload and Annotate Station Map (Supabase Storage)
// ════════════════════════════════════════════════════════════════════════════════

/// Upload a station map image to Supabase Storage.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_upload_map(
    state: State<'_, AppState>,
    station_id: Uuid,
    file_bytes: Vec<u8>,
    filename: String,
    content_type: String,
    image_width: Option<f64>,
    image_height: Option<f64>,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let map_uuid = Uuid::new_v4();
    let ext = filename.rsplit('.').next().unwrap_or("png");
    let storage_path = format!("station-maps/{}/{}.{}", station_id, map_uuid, ext);

    // Ensure rusa-maps bucket exists (idempotent)
    let client = reqwest::Client::new();
    let _ = client
        .post(&format!("{}/bucket", state.supabase_storage_url))
        .header(
            "Authorization",
            format!("Bearer {}", state.supabase_service_jwt),
        )
        .header("Content-Type", "application/json")
        .body(r#"{"id":"rusa-maps","name":"rusa-maps","public":false}"#)
        .send()
        .await;

    // Upload to Supabase Storage
    let upload_url = format!(
        "{}/object/rusa-maps/{}",
        state.supabase_storage_url, storage_path
    );

    let resp = client
        .post(&upload_url)
        .header(
            "Authorization",
            format!("Bearer {}", state.supabase_service_jwt),
        )
        .header("Content-Type", &content_type)
        .body(file_bytes)
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

    // Insert map record
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO station_maps
          (id, station_id, created_by, image_storage_path, image_width, image_height)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(map_uuid)
    .bind(station_id)
    .bind(user.id)
    .bind(&storage_path)
    .bind(image_width)
    .bind(image_height)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "station_maps",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "station_id": station_id,
            "storage_path": storage_path,
        })),
    )
    .await?;

    Ok(row.0)
}

/// Get all maps for a station (settler view — includes unpublished).
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_get_maps(
    state: State<'_, AppState>,
    station_id: Uuid,
) -> Result<Vec<StationMapSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let maps = sqlx::query_as::<_, StationMapSummary>(
        r#"
        SELECT id, station_id, image_storage_path, image_width, image_height,
               is_published, published_at, created_at
        FROM station_maps
        WHERE station_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(station_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(maps)
}

/// Publish a station map (makes it visible to Visitors).
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_publish_map(
    state: State<'_, AppState>,
    map_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    // Get the map's station_id
    let map_info: Option<(Uuid, Uuid)> = sqlx::query_as(
        r#"SELECT id, station_id FROM station_maps WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(map_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (_, station_id) = map_info
        .ok_or_else(|| AppError::Internal("Map not found.".into()))?;

    // Unpublish all other maps for this station
    sqlx::query(
        r#"
        UPDATE station_maps SET is_published = FALSE, published_at = NULL
        WHERE station_id = $1 AND id != $2 AND deleted_at IS NULL
        "#,
    )
    .bind(station_id)
    .bind(map_id)
    .execute(&state.db_pool)
    .await?;

    // Publish this one
    sqlx::query(
        r#"
        UPDATE station_maps SET is_published = TRUE, published_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(map_id)
    .execute(&state.db_pool)
    .await?;

    // Invalidate cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .del(format!("station_map:published:{}", station_id))
            .await;
    }

    write_audit_log(
        &state.db_pool,
        "station_maps",
        map_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({"is_published": false})),
        Some(serde_json::json!({"is_published": true})),
    )
    .await?;

    Ok(())
}

/// Add an annotation to a map.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_add_annotation(
    state: State<'_, AppState>,
    payload: AddAnnotationPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO station_map_annotations (map_id, label, description, x_position, y_position)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(payload.map_id)
    .bind(&payload.label)
    .bind(&payload.description)
    .bind(payload.x_position)
    .bind(payload.y_position)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "station_map_annotations",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "map_id": payload.map_id,
            "label": payload.label,
            "x": payload.x_position,
            "y": payload.y_position,
        })),
    )
    .await?;

    Ok(row.0)
}

/// Delete an annotation from a map.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_delete_annotation(
    state: State<'_, AppState>,
    annotation_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    sqlx::query(
        r#"
        UPDATE station_map_annotations
        SET deleted_at = NOW(), deleted_by = $1
        WHERE id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(user.id)
    .bind(annotation_id)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "station_map_annotations",
        annotation_id,
        AuditOperation::Delete,
        user.id,
        None,
        None,
    )
    .await?;

    Ok(())
}

/// Get annotations for a map.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_get_annotations(
    state: State<'_, AppState>,
    map_id: Uuid,
) -> Result<Vec<MapAnnotation>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let annotations = sqlx::query_as::<_, MapAnnotation>(
        r#"
        SELECT id, map_id, label, description, x_position, y_position
        FROM station_map_annotations
        WHERE map_id = $1 AND deleted_at IS NULL
        ORDER BY label
        "#,
    )
    .bind(map_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(annotations)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSV-01: View Interactive Station Map — Visitor (NO AUTH REQUIRED)
// ════════════════════════════════════════════════════════════════════════════════

/// Get the published station map with a signed URL from Supabase Storage.
/// This command does NOT require authentication — it's for Visitors.
///
/// **Access:** Public (no auth)
#[tauri::command]
pub async fn sst_get_published_map(
    state: State<'_, AppState>,
    station_id: Uuid,
) -> Result<PublishedMapData, AppError> {
    // NO AUTH — public endpoint for visitors

    // Check Redis cache (1 hour TTL)
    let cache_key = format!("station_map:published:{}", station_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(data) = serde_json::from_str::<PublishedMapData>(&cached) {
                return Ok(data);
            }
        }
    }

    // Find the published map
    let map = sqlx::query_as::<_, StationMapSummary>(
        r#"
        SELECT id, station_id, image_storage_path, image_width, image_height,
               is_published, published_at, created_at
        FROM station_maps
        WHERE station_id = $1 AND is_published = TRUE AND deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(station_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("No published map found for this station.".into()))?;

    // Get station name
    let station_name: (String,) = sqlx::query_as(
        r#"SELECT name FROM space_stations WHERE id = $1"#,
    )
    .bind(station_id)
    .fetch_one(&state.db_pool)
    .await?;

    // Get annotations
    let annotations = sqlx::query_as::<_, MapAnnotation>(
        r#"
        SELECT id, map_id, label, description, x_position, y_position
        FROM station_map_annotations
        WHERE map_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(map.id)
    .fetch_all(&state.db_pool)
    .await?;

    // Generate signed URL from Supabase Storage (time-limited 1 hour)
    let client = reqwest::Client::new();
    let sign_url = format!(
        "{}/object/sign/rusa-maps/{}",
        state.supabase_storage_url, map.image_storage_path
    );

    let sign_resp = client
        .post(&sign_url)
        .header(
            "Authorization",
            format!("Bearer {}", state.supabase_service_jwt),
        )
        .header("Content-Type", "application/json")
        .body(r#"{"expiresIn":3600}"#)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to generate signed URL: {}", e)))?;

    let signed_url = if sign_resp.status().is_success() {
        let body_text = sign_resp
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read signed URL response: {}", e)))?;
        let body: serde_json::Value = serde_json::from_str(&body_text)
            .map_err(|e| AppError::Internal(format!("Failed to parse signed URL response: {}", e)))?;
        let token = body["signedURL"]
            .as_str()
            .unwrap_or("");
        if token.starts_with("http") {
            token.to_string()
        } else {
            format!("{}{}", state.supabase_storage_url, token)
        }
    } else {
        // Fallback: return unsigned path (will fail for private buckets)
        format!(
            "{}/object/rusa-maps/{}",
            state.supabase_storage_url, map.image_storage_path
        )
    };

    let result = PublishedMapData {
        signed_url,
        image_width: map.image_width,
        image_height: map.image_height,
        annotations,
        station_name: station_name.0,
    };

    // Populate cache — 1 hour (matches signed URL expiry)
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&result) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 3600).await;
        }
    }

    Ok(result)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSS-06: Log Personnel On Board
// ════════════════════════════════════════════════════════════════════════════════

/// Log a personnel arrival at the station.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_log_arrival(
    state: State<'_, AppState>,
    payload: LogArrivalPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let arrived = DateTime::parse_from_rfc3339(&payload.arrived_at)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| AppError::Internal("Invalid arrived_at date format.".into()))?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO station_personnel_log
          (station_id, person_name, role, arrived_at, logged_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(payload.station_id)
    .bind(&payload.person_name)
    .bind(&payload.role)
    .bind(arrived)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Invalidate cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .del(format!("station_personnel:{}", payload.station_id))
            .await;
    }

    write_audit_log(
        &state.db_pool,
        "station_personnel_log",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "person_name": payload.person_name,
            "arrived_at": payload.arrived_at,
        })),
    )
    .await?;

    Ok(row.0)
}

/// Log a personnel departure from the station.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_log_departure(
    state: State<'_, AppState>,
    log_id: Uuid,
    departed_at: String,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let departed = DateTime::parse_from_rfc3339(&departed_at)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| AppError::Internal("Invalid departed_at date format.".into()))?;

    // Get current state for audit
    let current: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT station_id FROM station_personnel_log WHERE id = $1 AND departed_at IS NULL"#,
    )
    .bind(log_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (station_id,) = current
        .ok_or_else(|| AppError::Internal("Personnel log entry not found or already departed.".into()))?;

    sqlx::query(
        r#"UPDATE station_personnel_log SET departed_at = $1 WHERE id = $2"#,
    )
    .bind(departed)
    .bind(log_id)
    .execute(&state.db_pool)
    .await?;

    // Invalidate cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .del(format!("station_personnel:{}", station_id))
            .await;
    }

    write_audit_log(
        &state.db_pool,
        "station_personnel_log",
        log_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({"departed_at": null})),
        Some(serde_json::json!({"departed_at": departed_at})),
    )
    .await?;

    Ok(())
}

/// Get current personnel on board (active = no departure date).
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_get_personnel(
    state: State<'_, AppState>,
    station_id: Uuid,
    show_all: Option<bool>,
) -> Result<Vec<PersonnelLogEntry>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    // Check Redis cache (10 min TTL) — only for active personnel (default view)
    let use_cache = !show_all.unwrap_or(false);
    let cache_key = format!("station_personnel:{}", station_id);
    if use_cache {
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
                if let Ok(items) = serde_json::from_str::<Vec<PersonnelLogEntry>>(&cached) {
                    return Ok(items);
                }
            }
        }
    }

    let entries = if show_all.unwrap_or(false) {
        sqlx::query_as::<_, PersonnelLogEntry>(
            r#"
            SELECT id, station_id, person_name, role, arrived_at, departed_at, logged_by, created_at
            FROM station_personnel_log
            WHERE station_id = $1
            ORDER BY arrived_at DESC
            "#,
        )
        .bind(station_id)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, PersonnelLogEntry>(
            r#"
            SELECT id, station_id, person_name, role, arrived_at, departed_at, logged_by, created_at
            FROM station_personnel_log
            WHERE station_id = $1 AND departed_at IS NULL
            ORDER BY arrived_at DESC
            "#,
        )
        .bind(station_id)
        .fetch_all(&state.db_pool)
        .await?
    };

    // Populate cache — 10 minutes (active personnel only)
    if use_cache {
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(json) = serde_json::to_string(&entries) {
                let _: Result<(), _> = conn.set_ex(&cache_key, &json, 600).await;
            }
        }
    }

    Ok(entries)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-SSS-07: Propose Station Abandonment
// ════════════════════════════════════════════════════════════════════════════════

/// Propose abandoning the station. Creates a vote session for Directors.
/// Requires linking to a finding/archive report.
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_propose_abandonment(
    state: State<'_, AppState>,
    payload: ProposeAbandonmentPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    // Get station name
    let station_name: (String,) = sqlx::query_as(
        r#"SELECT name FROM space_stations WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(payload.station_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| AppError::Internal("Station not found.".into()))?;

    // Create vote session
    let vote_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions
          (topic, context, status, opens_at, created_by,
           total_yay, total_nay, total_abstain,
           admin_overridden, admin_terminated)
        VALUES ($1, $2, 'open', NOW(), $3, 0, 0, 0, FALSE, FALSE)
        RETURNING id
        "#,
    )
    .bind(format!("Station Abandonment — {}", station_name.0))
    .bind(format!(
        "Station abandonment proposed by {}. Reason: {}{}",
        user.full_name,
        payload.reason,
        payload
            .finding_reference_id
            .map(|id| format!(" (Finding ref: {})", id))
            .unwrap_or_default()
    ))
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify all Directors
    let directors: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name IN (
          'GeneralDirector','TheDirector','TheAccountant','TheLibrarian','TheNomad',
          'TheArtificer','TheObserver','TheWanderer','TheTaskmaster','TheGuardian',
          'TheStatistician','TheCoordinator','TheOverseer','TheAnchorman'
        ) AND u.deleted_at IS NULL AND u.is_active = true
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (dir_id,) in &directors {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'vote:new', $2)"#,
        )
        .bind(dir_id)
        .bind(serde_json::json!({
            "topic": format!("Station Abandonment — {}", station_name.0),
            "session_id": vote_row.0,
            "requester": user.full_name,
        }))
        .execute(&state.db_pool)
        .await;
    }

    write_audit_log(
        &state.db_pool,
        "space_stations",
        payload.station_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "action": "abandonment_proposed",
            "vote_session_id": vote_row.0,
            "reason": payload.reason,
        })),
    )
    .await?;

    Ok(vote_row.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// Helper: Get station list
// ════════════════════════════════════════════════════════════════════════════════

/// Get all active stations (used by layout to select station).
///
/// **Access:** SpaceStationSettler, Administrator
#[tauri::command]
pub async fn sst_get_stations(
    state: State<'_, AppState>,
) -> Result<Vec<StationSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::SpaceStationSettler, Role::Administrator]);

    let stations = sqlx::query_as::<_, StationSummary>(
        r#"
        SELECT id, name, sector, status, nearest_galactic_security_team_id, created_at
        FROM space_stations
        WHERE deleted_at IS NULL
        ORDER BY name
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(stations)
}

/// Get all active stations publicly (for visitor navigation). No auth required.
#[tauri::command]
pub async fn sst_get_public_stations(
    state: State<'_, AppState>,
) -> Result<Vec<StationSummary>, AppError> {
    let stations = sqlx::query_as::<_, StationSummary>(
        r#"
        SELECT id, name, sector, status, nearest_galactic_security_team_id, created_at
        FROM space_stations
        WHERE status = 'active' AND deleted_at IS NULL
        ORDER BY name
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(stations)
}

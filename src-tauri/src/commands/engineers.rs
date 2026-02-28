// commands/engineers.rs — Engineers subsystem commands
// Source of truth: 01_ENGINEERS.md, 00_MASTER_GUIDE.md
//
// Use cases implemented:
// - UC-GE-01: Receive Task Assignment (eng_get_my_tasks)
// - UC-GE-02: Submit Progress Report (eng_submit_progress_report)
// - UC-GE-03: Submit Cross-Department Help Request (eng_submit_help_request)
// - UC-AGE-01 / UC-BE-01: View Approved Test Database (eng_get_approved_tests)
// - UC-AGE-02 / UC-BE-03: Request New Test Approval (eng_propose_new_test)
// - UC-AGE-03 / UC-BE-04: Log Daily Experiment Findings (eng_log_daily_experiment)
// - UC-AGE-04 / UC-BE-05: Submit Experiment Conclusion (eng_submit_experiment_conclusion)
// - UC-AGE-05 / UC-BE-01: View Species Archive (eng_get_species_archive)
// - UC-AGE-06 / UC-BE-02: View Experiment Archive (eng_get_experiment_archive)
// - UC-ANC-00: Submit Broadcast Request (reuses submit_broadcast_request from directors.rs)
//
// All commands: authenticate, check role, execute, audit, invalidate Redis.

use chrono::{DateTime, NaiveDate, Utc};
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

// ── Engineer roles ────────────────────────────────────────────────────────────

const ENGINEER_ROLES: &[Role] = &[Role::AgriculturalEngineer, Role::BiologicalEngineer];

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EngineerTask {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    #[sqlx(rename = "type")]
    pub task_type: String,
    pub status: String,
    pub assigned_by: Uuid,
    pub assigner_name: String,
    pub due_date: Option<NaiveDate>,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitProgressReportPayload {
    pub task_id: Uuid,
    pub current_status: String,
    pub work_completed: String,
    pub problems_encountered: Option<String>,
    pub plans_next: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EngHelpRequestPayload {
    pub title: String,
    pub description: String,
    pub target_department: Option<String>,
    pub urgency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EngProposeTestPayload {
    pub name: String,
    pub goal: String,
    pub procedure: Option<String>,
    pub species_scope: Option<String>,
    pub category: Vec<String>,
    pub apparatuses: Option<String>,
    pub required_data: Option<String>,
    pub justification: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EngLogDailyPayload {
    pub experiment_id: Uuid,
    pub log_date: NaiveDate,
    pub rag_status: Option<String>,
    pub completed_actions: Option<String>,
    pub pending_actions: Option<String>,
    pub collected_data: Option<serde_json::Value>,
    pub attendee_ids: Option<Vec<Uuid>>,
    pub test_ids: Option<Vec<Uuid>>,
    pub species_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct EngConclusionPayload {
    pub experiment_id: Uuid,
    pub summary: String,
    pub final_outcomes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ApprovedTestRow {
    pub id: Uuid,
    pub name: String,
    pub procedure: String,
    pub category: String,
    pub applicable_scope: String,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SpeciesArchiveRow {
    pub id: Uuid,
    #[sqlx(rename = "type")]
    pub archive_type: String,
    pub name: String,
    pub classification: Option<String>,
    pub detail: serde_json::Value,
    pub experiment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EngExperimentSummary {
    pub id: Uuid,
    pub title: String,
    #[sqlx(rename = "type")]
    pub experiment_type: String,
    pub status: String,
    pub proposed_by: Uuid,
    pub proposer_name: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EngExperimentLogSummary {
    pub id: Uuid,
    pub experiment_id: Uuid,
    pub log_date: NaiveDate,
    pub rag_status: Option<String>,
    pub completed_actions: Option<String>,
    pub pending_actions: Option<String>,
    pub collected_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProgressReportRow {
    pub id: Uuid,
    pub task_id: Uuid,
    pub submitted_by: Uuid,
    pub submitter_name: String,
    pub report_date: NaiveDate,
    pub current_status: String,
    pub work_completed: String,
    pub problems_encountered: Option<String>,
    pub plans_next: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns the experiment type(s) allowed for the given engineer role.
fn role_to_experiment_types(role: &Role) -> &'static [&'static str] {
    match role {
        Role::AgriculturalEngineer => &["agricultural"],
        Role::BiologicalEngineer => &["biological_engineering"],
        _ => &["agricultural", "biological_engineering"], // admin
    }
}

/// Returns the approved test scope for the given engineer role.
fn role_to_test_scope(role: &Role) -> &'static str {
    match role {
        Role::AgriculturalEngineer => "plants",
        Role::BiologicalEngineer => "all_species",
        _ => "all_species", // admin default
    }
}

/// Returns the species archive scope for the given engineer role.
fn role_to_species_scope(role: &Role) -> &'static str {
    match role {
        Role::AgriculturalEngineer => "plant",
        _ => "all_species",
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-GE-01: Get My Tasks (task inbox)
// ════════════════════════════════════════════════════════════════════════════════

/// Retrieve tasks assigned to the current engineer.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_get_my_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<EngineerTask>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    // Check Redis cache
    let cache_key = format!("engineer_tasks:{}", user.id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(items) = serde_json::from_str::<Vec<EngineerTask>>(&cached) {
                return Ok(items);
            }
        }
    }

    let tasks = sqlx::query_as::<_, EngineerTask>(
        r#"
        SELECT t.id, t.title, t.description, t.type, t.status,
               t.assigned_by, u.full_name AS assigner_name,
               t.due_date, t.payload, t.created_at
        FROM tasks t
        JOIN users u ON u.id = t.assigned_by
        WHERE t.assigned_to = $1 AND t.deleted_at IS NULL
        ORDER BY t.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    // Cache for 15 minutes
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&tasks) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 900).await;
        }
    }

    Ok(tasks)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-GE-02: Submit Progress Report
// ════════════════════════════════════════════════════════════════════════════════

/// Submit a progress report linked to an active task.
/// Routes report to The Observer.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_submit_progress_report(
    state: State<'_, AppState>,
    payload: SubmitProgressReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    if payload.current_status.trim().is_empty() || payload.work_completed.trim().is_empty() {
        return Err(AppError::Internal("Status and work completed are required.".into()));
    }

    // Verify task exists and is assigned to this user
    let task_check: Option<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, title FROM tasks
        WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.task_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (_, task_title) = task_check
        .ok_or_else(|| AppError::Internal("Task not found or not assigned to you.".into()))?;

    let report_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO progress_reports (task_id, submitted_by, current_status, work_completed, problems_encountered, plans_next)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(payload.task_id)
    .bind(user.id)
    .bind(&payload.current_status)
    .bind(&payload.work_completed)
    .bind(&payload.problems_encountered)
    .bind(&payload.plans_next)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify The Observer
    let observer_ids: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'TheObserver' AND u.deleted_at IS NULL
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (obs_id,) in &observer_ids {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'progress_report:submitted', $2::jsonb)
            "#,
        )
        .bind(obs_id)
        .bind(serde_json::json!({
            "report_id": report_id.0,
            "task_id": payload.task_id,
            "task_title": task_title,
            "submitted_by": user.full_name,
            "status": payload.current_status,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "progress_reports",
        report_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "task_id": payload.task_id,
            "current_status": payload.current_status,
        })),
    )
    .await?;

    // Invalidate task cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("engineer_tasks:{}", user.id)).await;
    }

    Ok(report_id.0)
}

/// Get progress reports for a specific task.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_get_progress_reports(
    state: State<'_, AppState>,
    task_id: Uuid,
) -> Result<Vec<ProgressReportRow>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    let reports = sqlx::query_as::<_, ProgressReportRow>(
        r#"
        SELECT pr.id, pr.task_id, pr.submitted_by, u.full_name AS submitter_name,
               pr.report_date, pr.current_status, pr.work_completed,
               pr.problems_encountered, pr.plans_next, pr.created_at
        FROM progress_reports pr
        JOIN users u ON u.id = pr.submitted_by
        WHERE pr.task_id = $1 AND pr.deleted_at IS NULL
        ORDER BY pr.created_at DESC
        "#,
    )
    .bind(task_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(reports)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-GE-03: Submit Cross-Department Help Request
// ════════════════════════════════════════════════════════════════════════════════

/// Submit a cross-department help request. Routes to The Observer (proxy).
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_submit_help_request(
    state: State<'_, AppState>,
    payload: EngHelpRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    if payload.title.trim().is_empty() {
        return Err(AppError::Internal("Help request title cannot be empty.".into()));
    }

    // Both Agricultural and Biological Engineers route to The Observer
    let proxy_role = "TheObserver";

    let proxy_ids: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(proxy_role)
    .fetch_all(&state.db_pool)
    .await?;

    if proxy_ids.is_empty() {
        return Err(AppError::Internal("No active Observer found.".into()));
    }

    let proxy_id = proxy_ids[0].0;

    let help_payload = serde_json::json!({
        "target_department": payload.target_department,
        "urgency": payload.urgency,
        "requester_role": format!("{:?}", user.role),
    });

    let task_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO tasks (assigned_by, assigned_to, type, title, description, payload)
        VALUES ($1, $2, 'help_request', $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(proxy_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&help_payload)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify proxy director
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'help_request:received', $2::jsonb)
        "#,
    )
    .bind(proxy_id)
    .bind(serde_json::json!({
        "task_id": task_id.0,
        "title": payload.title,
        "from": user.full_name,
        "requester_role": format!("{:?}", user.role),
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "tasks",
        task_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "type": "help_request",
            "title": payload.title,
            "proxy_director": proxy_role,
        })),
    )
    .await?;

    Ok(task_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AGE-01 / UC-BE-01: View Approved Test Database
// ════════════════════════════════════════════════════════════════════════════════

/// Get approved tests filtered by engineer's applicable scope.
/// Agricultural → 'plants', Biological → 'all_species'.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_get_approved_tests(
    state: State<'_, AppState>,
    scope: Option<String>,
) -> Result<Vec<ApprovedTestRow>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    let effective_scope = scope.unwrap_or_else(|| role_to_test_scope(&user.role).to_string());

    // Check Redis cache
    let cache_key = format!("approved_tests:{}", effective_scope);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(items) = serde_json::from_str::<Vec<ApprovedTestRow>>(&cached) {
                return Ok(items);
            }
        }
    }

    let tests = sqlx::query_as::<_, ApprovedTestRow>(
        r#"
        SELECT id, name, procedure, category, applicable_scope, accepted_at, created_at
        FROM approved_tests
        WHERE applicable_scope = $1 AND deleted_at IS NULL
        ORDER BY name
        "#,
    )
    .bind(&effective_scope)
    .fetch_all(&state.db_pool)
    .await?;

    // Cache for 1 hour
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&tests) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 3600).await;
        }
    }

    Ok(tests)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AGE-02 / UC-BE-03: Request New Test Approval
// ════════════════════════════════════════════════════════════════════════════════

/// Propose a new test method. Routes to The Observer for approval.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_propose_new_test(
    state: State<'_, AppState>,
    payload: EngProposeTestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    if payload.name.trim().is_empty() {
        return Err(AppError::Internal("Test name cannot be empty.".into()));
    }

    let proposal_data = serde_json::json!({
        "name": payload.name,
        "goal": payload.goal,
        "procedure": payload.procedure,
        "species_scope": payload.species_scope.clone().unwrap_or_else(|| role_to_test_scope(&user.role).to_string()),
        "category": payload.category,
        "apparatuses": payload.apparatuses,
        "required_data": payload.required_data,
        "justification": payload.justification,
    });

    let prop_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO test_proposals (proposed_by, proposal_data)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&proposal_data)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify The Observer
    let observer_ids: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'TheObserver' AND u.deleted_at IS NULL
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (obs_id,) in &observer_ids {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'test_proposal:submitted', $2::jsonb)
            "#,
        )
        .bind(obs_id)
        .bind(serde_json::json!({
            "proposal_id": prop_id.0,
            "test_name": payload.name,
            "proposed_by": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "test_proposals",
        prop_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "name": payload.name })),
    )
    .await?;

    Ok(prop_id.0)
}

/// Get test proposals submitted by the current engineer.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_get_my_test_proposals(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    let rows: Vec<(Uuid, serde_json::Value, String, Option<String>, DateTime<Utc>)> =
        sqlx::query_as(
            r#"
            SELECT id, proposal_data, status, reviewer_note, created_at
            FROM test_proposals
            WHERE proposed_by = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.db_pool)
        .await?;

    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, data, status, note, created)| {
            serde_json::json!({
                "id": id,
                "proposal_data": data,
                "status": status,
                "reviewer_note": note,
                "created_at": created,
            })
        })
        .collect();

    Ok(result)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AGE-03 / UC-BE-04: Log Daily Experiment Findings
// ════════════════════════════════════════════════════════════════════════════════

/// Log a daily experiment session linked to an active experiment.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_log_daily_experiment(
    state: State<'_, AppState>,
    payload: EngLogDailyPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    // Validate RAG status if provided
    if let Some(ref rag) = payload.rag_status {
        if !["red", "amber", "green"].contains(&rag.as_str()) {
            return Err(AppError::Internal("RAG status must be red, amber, or green.".into()));
        }
    }

    // Verify experiment exists and is active/approved
    let exp_check: Option<(Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT id, type, status FROM experiments
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.experiment_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (_, exp_type, exp_status) = exp_check
        .ok_or_else(|| AppError::Internal("Experiment not found.".into()))?;

    if exp_status != "approved" && exp_status != "active" {
        return Err(AppError::Internal(
            "Can only log sessions for approved or active experiments.".into(),
        ));
    }

    // Validate experiment type matches role
    let allowed_types = role_to_experiment_types(&user.role);
    if !allowed_types.contains(&exp_type.as_str()) {
        return Err(AppError::Forbidden);
    }

    // Update experiment status to 'active' if still 'approved'
    if exp_status == "approved" {
        sqlx::query(
            "UPDATE experiments SET status = 'active', updated_at = NOW() WHERE id = $1",
        )
        .bind(payload.experiment_id)
        .execute(&state.db_pool)
        .await?;
    }

    let log_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO experiment_daily_logs
          (experiment_id, log_date, rag_status, completed_actions, pending_actions, collected_data, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(payload.experiment_id)
    .bind(payload.log_date)
    .bind(&payload.rag_status)
    .bind(&payload.completed_actions)
    .bind(&payload.pending_actions)
    .bind(&payload.collected_data)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Insert attendees
    if let Some(ref attendees) = payload.attendee_ids {
        for attendee_id in attendees {
            sqlx::query(
                "INSERT INTO experiment_log_attendees (log_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(log_id.0)
            .bind(attendee_id)
            .execute(&state.db_pool)
            .await?;
        }
    }

    // Insert tests performed
    if let Some(ref tests) = payload.test_ids {
        for test_id in tests {
            sqlx::query(
                "INSERT INTO experiment_log_tests (log_id, test_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(log_id.0)
            .bind(test_id)
            .execute(&state.db_pool)
            .await?;
        }
    }

    // Insert species worked on
    if let Some(ref species) = payload.species_ids {
        for sp_id in species {
            sqlx::query(
                "INSERT INTO experiment_log_species (log_id, species_archive_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(log_id.0)
            .bind(sp_id)
            .execute(&state.db_pool)
            .await?;
        }
    }

    write_audit_log(
        &state.db_pool,
        "experiment_daily_logs",
        log_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "experiment_id": payload.experiment_id,
            "log_date": payload.log_date.to_string(),
        })),
    )
    .await?;

    // Invalidate cached experiments for this user
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("experiments:user:{}", user.id)).await;
    }

    Ok(log_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AGE-04 / UC-BE-05: Submit Experiment Conclusion Request
// ════════════════════════════════════════════════════════════════════════════════

/// Submit an experiment conclusion request. Routes to The Taskmaster.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_submit_experiment_conclusion(
    state: State<'_, AppState>,
    payload: EngConclusionPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    // Verify experiment exists, is active, and matches role
    let exp: Option<(Uuid, String, String, String)> = sqlx::query_as(
        r#"
        SELECT e.id, e.type, e.status, e.title FROM experiments e
        WHERE e.id = $1 AND e.deleted_at IS NULL
        "#,
    )
    .bind(payload.experiment_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (_, exp_type, exp_status, exp_title) = exp
        .ok_or_else(|| AppError::Internal("Experiment not found.".into()))?;

    if exp_status != "active" {
        return Err(AppError::Internal(
            "Can only request conclusion for active experiments.".into(),
        ));
    }

    let allowed_types = role_to_experiment_types(&user.role);
    if !allowed_types.contains(&exp_type.as_str()) {
        return Err(AppError::Forbidden);
    }

    // Update experiment status
    sqlx::query(
        "UPDATE experiments SET status = 'conclusion_requested', updated_at = NOW() WHERE id = $1",
    )
    .bind(payload.experiment_id)
    .execute(&state.db_pool)
    .await?;

    // Create a request routed to The Taskmaster
    let req_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, requester_id, title, description, payload, bypass_authority)
        VALUES ('experiment_closure', $1, $2, $3, $4, 'TheTaskmaster')
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(format!("Conclusion: {}", exp_title))
    .bind(&payload.summary)
    .bind(serde_json::json!({
        "experiment_id": payload.experiment_id,
        "experiment_type": exp_type,
        "final_outcomes": payload.final_outcomes,
    }))
    .fetch_one(&state.db_pool)
    .await?;

    // Notify The Taskmaster
    let taskmasters: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'TheTaskmaster' AND u.deleted_at IS NULL
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (tm_id,) in &taskmasters {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'experiment:conclusion_requested', $2::jsonb)
            "#,
        )
        .bind(tm_id)
        .bind(serde_json::json!({
            "request_id": req_id.0,
            "experiment_id": payload.experiment_id,
            "title": exp_title,
            "requested_by": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "requests",
        req_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "type": "experiment_closure",
            "experiment_id": payload.experiment_id,
        })),
    )
    .await?;

    // Invalidate cached experiments for this user
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("experiments:user:{}", user.id)).await;
    }

    Ok(req_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AGE-05 / UC-BE-01: View Species Archive
// ════════════════════════════════════════════════════════════════════════════════

/// Get species archive entries. Agricultural → scope='plant' (species only),
/// Biological → all species from science_archive with type='species'.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_get_species_archive(
    state: State<'_, AppState>,
    search: Option<String>,
) -> Result<Vec<SpeciesArchiveRow>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    // For Agricultural Engineers, filter species by plant-related scope
    // For Biological Engineers, show all species
    let is_agri = user.role == Role::AgriculturalEngineer;

    // Check Redis cache (only for unfiltered queries)
    if search.is_none() {
        let scope = role_to_species_scope(&user.role);
        let cache_key = format!("species_archive:{}", scope);
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
                if let Ok(items) = serde_json::from_str::<Vec<SpeciesArchiveRow>>(&cached) {
                    return Ok(items);
                }
            }
        }
    }

    let items = if is_agri {
        // Agricultural: only species with plant-related classification
        if let Some(ref q) = search {
            let pattern = format!("%{}%", q);
            sqlx::query_as::<_, SpeciesArchiveRow>(
                r#"
                SELECT id, type, name, classification, detail, experiment_id, created_at
                FROM science_archive
                WHERE type = 'species' AND deleted_at IS NULL
                  AND (detail->>'taxonomy')::jsonb->>'kingdom' = 'Plantae'
                  AND (LOWER(name) LIKE LOWER($1) OR LOWER(classification) LIKE LOWER($1))
                ORDER BY name
                "#,
            )
            .bind(&pattern)
            .fetch_all(&state.db_pool)
            .await
            .unwrap_or_else(|_| {
                // Fallback: just filter by name if JSONB path fails
                vec![]
            })
        } else {
            // For plant scope, use a simpler filter — include all species type entries
            // When taxonomy data exists, filter to plant kingdom; otherwise show all
            sqlx::query_as::<_, SpeciesArchiveRow>(
                r#"
                SELECT id, type, name, classification, detail, experiment_id, created_at
                FROM science_archive
                WHERE type = 'species' AND deleted_at IS NULL
                ORDER BY name
                "#,
            )
            .fetch_all(&state.db_pool)
            .await?
        }
    } else {
        // Biological: all species
        if let Some(ref q) = search {
            let pattern = format!("%{}%", q);
            sqlx::query_as::<_, SpeciesArchiveRow>(
                r#"
                SELECT id, type, name, classification, detail, experiment_id, created_at
                FROM science_archive
                WHERE type = 'species' AND deleted_at IS NULL
                  AND (LOWER(name) LIKE LOWER($1) OR LOWER(classification) LIKE LOWER($1))
                ORDER BY name
                "#,
            )
            .bind(&pattern)
            .fetch_all(&state.db_pool)
            .await?
        } else {
            sqlx::query_as::<_, SpeciesArchiveRow>(
                r#"
                SELECT id, type, name, classification, detail, experiment_id, created_at
                FROM science_archive
                WHERE type = 'species' AND deleted_at IS NULL
                ORDER BY name
                "#,
            )
            .fetch_all(&state.db_pool)
            .await?
        }
    };

    // Cache (unfiltered only)
    if search.is_none() {
        let scope = role_to_species_scope(&user.role);
        let cache_key = format!("species_archive:{}", scope);
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(json) = serde_json::to_string(&items) {
                let _: Result<(), _> = conn.set_ex(&cache_key, &json, 3600).await;
            }
        }
    }

    Ok(items)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-AGE-06 / UC-BE-02: View Experiment Archive
// ════════════════════════════════════════════════════════════════════════════════

/// Get experiment archive filtered by engineer's discipline.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_get_experiment_archive(
    state: State<'_, AppState>,
    status_filter: Option<String>,
) -> Result<Vec<EngExperimentSummary>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    let exp_types = role_to_experiment_types(&user.role);

    let experiments = if let Some(ref status) = status_filter {
        sqlx::query_as::<_, EngExperimentSummary>(
            r#"
            SELECT e.id, e.title, e.type, e.status, e.proposed_by,
                   u.full_name AS proposer_name, e.metadata, e.created_at,
                   e.approved_at, e.closed_at
            FROM experiments e
            JOIN users u ON u.id = e.proposed_by
            WHERE e.type = ANY($1) AND e.status = $2 AND e.deleted_at IS NULL
            ORDER BY e.created_at DESC
            "#,
        )
        .bind(exp_types)
        .bind(status)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, EngExperimentSummary>(
            r#"
            SELECT e.id, e.title, e.type, e.status, e.proposed_by,
                   u.full_name AS proposer_name, e.metadata, e.created_at,
                   e.approved_at, e.closed_at
            FROM experiments e
            JOIN users u ON u.id = e.proposed_by
            WHERE e.type = ANY($1) AND e.deleted_at IS NULL
            ORDER BY e.created_at DESC
            "#,
        )
        .bind(exp_types)
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(experiments)
}

/// Get experiment detail including daily logs.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_get_experiment_detail(
    state: State<'_, AppState>,
    experiment_id: Uuid,
) -> Result<(EngExperimentSummary, Vec<EngExperimentLogSummary>), AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    let experiment = sqlx::query_as::<_, EngExperimentSummary>(
        r#"
        SELECT e.id, e.title, e.type, e.status, e.proposed_by,
               u.full_name AS proposer_name, e.metadata, e.created_at,
               e.approved_at, e.closed_at
        FROM experiments e
        JOIN users u ON u.id = e.proposed_by
        WHERE e.id = $1 AND e.deleted_at IS NULL
        "#,
    )
    .bind(experiment_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Experiment not found.".into()))?;

    let logs = sqlx::query_as::<_, EngExperimentLogSummary>(
        r#"
        SELECT id, experiment_id, log_date, rag_status, completed_actions,
               pending_actions, collected_data, created_at
        FROM experiment_daily_logs
        WHERE experiment_id = $1 AND deleted_at IS NULL
        ORDER BY log_date DESC
        "#,
    )
    .bind(experiment_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok((experiment, logs))
}

/// Get help request status for the current engineer.
///
/// **Access:** AgriculturalEngineer, BiologicalEngineer, or Administrator.
#[tauri::command]
pub async fn eng_get_my_help_requests(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::AgriculturalEngineer, Role::BiologicalEngineer
    ]);

    let rows: Vec<(Uuid, String, Option<String>, String, serde_json::Value, DateTime<Utc>)> =
        sqlx::query_as(
            r#"
            SELECT t.id, t.title, t.description, t.status, t.payload, t.created_at
            FROM tasks t
            WHERE t.assigned_by = $1 AND t.type = 'help_request' AND t.deleted_at IS NULL
            ORDER BY t.created_at DESC
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.db_pool)
        .await?;

    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, title, desc, status, payload, created)| {
            serde_json::json!({
                "id": id,
                "title": title,
                "description": desc,
                "status": status,
                "payload": payload,
                "created_at": created,
            })
        })
        .collect();

    Ok(result)
}

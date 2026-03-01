// commands/scientists.rs — Scientist subsystem commands
// Source of truth: 04_SCIENTISTS.md, 00_MASTER_GUIDE.md
//
// Use cases implemented:
// - UC-GS-01: Receive Task Assignment (get_my_tasks)
// - UC-GS-02: Submit Cross-Department Help Request (submit_help_request)
// - UC-GS-03: Request Data from Data Analysts (reuses submit_data_request from data_analysts.rs)
// - UC-MA-01: Submit Calculation / Results Response (submit_math_results)
// - UC-PH-01 / UC-CH-04 / UC-BIO-02: Propose Experiment (propose_experiment)
// - UC-PH-02 / UC-CH-05 / UC-BIO-03: Log Experiment / Observation (log_experiment_session)
// - UC-PH-07 / UC-CH-06 / UC-BIO-07: Submit Experiment Conclusion (submit_experiment_conclusion)
// - UC-PH-08 / UC-CH-07 / UC-BIO-04: Submit Final Document (submit_final_document)
// - UC-PH-03/06, UC-CH-01/01A, UC-BIO-01/01A: View Archives (get_archive, get_experiment_archive)
// - UC-PH-04 / UC-CH-02 / UC-BIO-05: View Approved Tests (get_approved_tests)
// - UC-PH-05 / UC-CH-03 / UC-BIO-06: Propose New Test (propose_new_test)
// - UC-BIO-02 (species-specific): Propose New Species (propose_new_species)

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

// ── Scientist roles ───────────────────────────────────────────────────────────

#[allow(dead_code)]
const SCIENTIST_ROLES: &[Role] = &[Role::Mathematician, Role::Physicist, Role::Chemist, Role::Biologist];
#[allow(dead_code)]
const EXPERIMENT_ROLES: &[Role] = &[Role::Physicist, Role::Chemist, Role::Biologist];

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, FromRow)]
pub struct ScientistTask {
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
pub struct SubmitHelpRequestPayload {
    pub title: String,
    pub description: String,
    pub urgency: Option<String>,
    pub calculations_area: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitMathResultsPayload {
    pub task_id: Uuid,
    pub content_latex: String,
    pub workings: Option<String>,
    pub calculations_area: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProposeExperimentPayload {
    pub title: String,
    pub experiment_type: String, // 'chemical', 'physical', 'biology_observation'
    pub introduction: Option<String>,
    pub problem_statement: Option<String>,
    pub research_questions: Option<String>,
    pub hypotheses: Option<String>,
    pub methodology: Option<String>,
    pub expected_outcomes: Option<String>,
    pub location: Option<String>,
    pub related_objects: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct LogExperimentSessionPayload {
    pub experiment_id: Uuid,
    pub log_date: NaiveDate,
    pub rag_status: Option<String>,
    pub completed_actions: Option<String>,
    pub pending_actions: Option<String>,
    pub collected_data: Option<serde_json::Value>,
    pub attendee_ids: Option<Vec<Uuid>>,
    pub test_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitExperimentConclusionPayload {
    pub experiment_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitFinalDocumentPayload {
    pub experiment_id: Uuid,
    pub doc_type: String, // 'matter', 'physical_object', 'species'
    pub document_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ArchiveItem {
    pub id: Uuid,
    #[sqlx(rename = "type")]
    pub archive_type: String,
    pub name: String,
    pub classification: Option<String>,
    pub detail: serde_json::Value,
    pub storage_path: Option<String>,
    pub experiment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ApprovedTest {
    pub id: Uuid,
    pub name: String,
    pub procedure: String,
    pub category: String,
    pub applicable_scope: String,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ProposeNewTestPayload {
    pub name: String,
    pub goal: String,
    pub procedure: String,
    pub species_scope: Option<String>,
    pub category: Vec<String>,
    pub apparatuses: Option<String>,
    pub required_data: Option<String>,
    pub justification: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProposeNewSpeciesPayload {
    pub title: String,
    pub introduction: Option<String>,
    pub problem_statement: Option<String>,
    pub research_questions: Option<String>,
    pub hypotheses: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExperimentSummary {
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
pub struct ExperimentLogSummary {
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
pub struct MathResult {
    pub id: Uuid,
    pub task_id: Uuid,
    pub submitted_by: Uuid,
    pub content: serde_json::Value,
    pub pdf_storage_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FinalDocSummary {
    pub id: Uuid,
    pub experiment_id: Uuid,
    #[sqlx(rename = "type")]
    pub doc_type: String,
    pub document_data: serde_json::Value,
    pub status: String,
    pub submitted_by: Uuid,
    pub created_at: DateTime<Utc>,
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-GS-01: Get My Tasks (task inbox)
// ════════════════════════════════════════════════════════════════════════════════

/// Retrieve tasks assigned to the current scientist.
///
/// **Access:** Mathematician, Physicist, Chemist, Biologist, or Administrator.
#[tauri::command]
pub async fn get_my_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<ScientistTask>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Mathematician, Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    let tasks = sqlx::query_as::<_, ScientistTask>(
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

    Ok(tasks)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-GS-02: Submit Cross-Department Help Request
// ════════════════════════════════════════════════════════════════════════════════

/// Submit a cross-department help request.
/// Routes to the proxy Director: Artificer (Math/Phys) or Observer (Chem/Bio).
///
/// **Access:** Physicist, Chemist, Biologist, or Administrator.
/// (Mathematician does NOT have cross-dept help requests per spec.)
#[tauri::command]
pub async fn submit_help_request(
    state: State<'_, AppState>,
    payload: SubmitHelpRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    if payload.title.trim().is_empty() {
        return Err(AppError::Internal("Help request title cannot be empty.".into()));
    }

    // Determine proxy director based on scientist role
    let proxy_role = match user.role {
        Role::Physicist => "TheArtificer",
        Role::Chemist | Role::Biologist => "TheObserver",
        _ => "TheObserver", // fallback (admin acting)
    };

    // Create a task of type 'help_request' assigned to proxy director
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
        return Err(AppError::Internal(format!("No active {} found.", proxy_role)));
    }

    let proxy_id = proxy_ids[0].0;

    let help_payload = serde_json::json!({
        "urgency": payload.urgency,
        "calculations_area": payload.calculations_area,
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
// UC-MA-01: Submit Calculation / Results Response
// ════════════════════════════════════════════════════════════════════════════════

/// Submit math results (LaTeX content) for an assigned task.
///
/// **Access:** Mathematician or Administrator.
#[tauri::command]
pub async fn submit_math_results(
    state: State<'_, AppState>,
    payload: SubmitMathResultsPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth!(state, Role::Mathematician);

    // Verify the task exists and is assigned to this user
    let task_check: Option<(Uuid, Uuid)> = sqlx::query_as(
        r#"
        SELECT id, assigned_by FROM tasks
        WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.task_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (_, assigner_id) = task_check
        .ok_or_else(|| AppError::Internal("Task not found or not assigned to you.".into()))?;

    let content = serde_json::json!({
        "content_latex": payload.content_latex,
        "workings": payload.workings,
        "calculations_area": payload.calculations_area,
    });

    let result_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO math_results (task_id, submitted_by, content)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(payload.task_id)
    .bind(user.id)
    .bind(&content)
    .fetch_one(&state.db_pool)
    .await?;

    // Mark task as completed
    sqlx::query(
        "UPDATE tasks SET status = 'completed', completed_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(payload.task_id)
    .execute(&state.db_pool)
    .await?;

    // Notify the assigning director (Artificer)
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'math_results:submitted', $2::jsonb)
        "#,
    )
    .bind(assigner_id)
    .bind(serde_json::json!({
        "result_id": result_id.0,
        "task_id": payload.task_id,
        "submitted_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "math_results",
        result_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "task_id": payload.task_id,
        })),
    )
    .await?;

    Ok(result_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-PH-01 / UC-CH-04 / UC-BIO-02: Propose Experiment
// ════════════════════════════════════════════════════════════════════════════════

/// Propose a new experiment. Enters standard approval workflow.
///
/// **Access:** Physicist, Chemist, Biologist, or Administrator.
#[tauri::command]
pub async fn propose_experiment(
    state: State<'_, AppState>,
    payload: ProposeExperimentPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    // Validate experiment type matches role
    let valid_types: &[&str] = match user.role {
        Role::Physicist => &["physical"],
        Role::Chemist => &["chemical"],
        Role::Biologist => &["biology_observation"],
        _ => &["physical", "chemical", "biology_observation"], // admin
    };

    if !valid_types.contains(&payload.experiment_type.as_str()) {
        return Err(AppError::Internal(format!(
            "Invalid experiment type '{}' for your role.", payload.experiment_type
        )));
    }

    let metadata = serde_json::json!({
        "introduction": payload.introduction,
        "problem_statement": payload.problem_statement,
        "research_questions": payload.research_questions,
        "hypotheses": payload.hypotheses,
        "methodology": payload.methodology,
        "expected_outcomes": payload.expected_outcomes,
        "related_objects": payload.related_objects,
        "location": payload.location,
    });

    let exp_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO experiments (title, type, metadata, proposed_by)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.experiment_type)
    .bind(&metadata)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Create a request for the approval workflow
    // Per spec: experiment proposals route to Directors for vote
    let bypass = match user.role {
        Role::Physicist | Role::Mathematician => "TheArtificer",
        Role::Chemist | Role::Biologist => "TheObserver",
        _ => "TheObserver",
    };

    let req_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, requester_id, title, description, payload, bypass_authority)
        VALUES ('experiment_proposal', $1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.title)
    .bind(payload.introduction.as_deref())
    .bind(serde_json::json!({
        "experiment_id": exp_id.0,
        "experiment_type": payload.experiment_type,
    }))
    .bind(bypass)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify the relevant director
    let director_ids: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(bypass)
    .fetch_all(&state.db_pool)
    .await?;

    for (dir_id,) in &director_ids {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'experiment:proposed', $2::jsonb)
            "#,
        )
        .bind(dir_id)
        .bind(serde_json::json!({
            "request_id": req_id.0,
            "experiment_id": exp_id.0,
            "title": payload.title,
            "proposed_by": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "experiments",
        exp_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "title": payload.title,
            "type": payload.experiment_type,
            "request_id": req_id.0,
        })),
    )
    .await?;

    Ok(exp_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-PH-02 / UC-CH-05 / UC-BIO-03: Log Experiment / Observation
// ════════════════════════════════════════════════════════════════════════════════

/// Log a daily experiment session linked to an active experiment.
///
/// **Access:** Physicist, Chemist, Biologist, or Administrator.
#[tauri::command]
pub async fn log_experiment_session(
    state: State<'_, AppState>,
    payload: LogExperimentSessionPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    // Validate RAG status if provided
    if let Some(ref rag) = payload.rag_status {
        if !["red", "amber", "green"].contains(&rag.as_str()) {
            return Err(AppError::Internal("RAG status must be red, amber, or green.".into()));
        }
    }

    // Verify experiment is active/approved and belongs to current user's discipline
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
    let allowed_types: &[&str] = match user.role {
        Role::Physicist => &["physical"],
        Role::Chemist => &["chemical"],
        Role::Biologist => &["biology_observation"],
        _ => &["physical", "chemical", "biology_observation"], // admin
    };

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

    Ok(log_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-PH-07 / UC-CH-06 / UC-BIO-07: Submit Experiment Conclusion
// ════════════════════════════════════════════════════════════════════════════════

/// Submit an experiment conclusion request. Routes to The Taskmaster.
///
/// **Access:** Physicist, Chemist, Biologist, or Administrator.
#[tauri::command]
pub async fn submit_experiment_conclusion(
    state: State<'_, AppState>,
    payload: SubmitExperimentConclusionPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    // Verify experiment exists, is active, and belongs to user's discipline
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
    .bind(&payload.reason)
    .bind(serde_json::json!({
        "experiment_id": payload.experiment_id,
        "experiment_type": exp_type,
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

    Ok(req_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-PH-08 / UC-CH-07 / UC-BIO-04: Submit Final Document
// ════════════════════════════════════════════════════════════════════════════════

/// Submit a final object/matter/species document post-experiment conclusion.
///
/// **Access:** Physicist, Chemist, Biologist, or Administrator.
#[tauri::command]
pub async fn submit_final_document(
    state: State<'_, AppState>,
    payload: SubmitFinalDocumentPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    // Validate doc_type
    let valid_types = ["matter", "physical_object", "species"];
    if !valid_types.contains(&payload.doc_type.as_str()) {
        return Err(AppError::Internal(
            "Document type must be 'matter', 'physical_object', or 'species'.".into(),
        ));
    }

    // Validate doc type matches role
    match user.role {
        Role::Physicist if payload.doc_type != "physical_object" => {
            return Err(AppError::Internal("Physicists can only submit physical_object documents.".into()));
        }
        Role::Chemist if payload.doc_type != "matter" => {
            return Err(AppError::Internal("Chemists can only submit matter documents.".into()));
        }
        Role::Biologist if payload.doc_type != "species" => {
            return Err(AppError::Internal("Biologists can only submit species documents.".into()));
        }
        _ => {}
    }

    // Verify experiment is closed
    let exp_check: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM experiments WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.experiment_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (status,) = exp_check
        .ok_or_else(|| AppError::Internal("Experiment not found.".into()))?;

    if status != "closed" {
        return Err(AppError::Internal(
            "Can only submit final documents for closed experiments.".into(),
        ));
    }

    let doc_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO final_object_documents (experiment_id, type, document_data, submitted_by)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(payload.experiment_id)
    .bind(&payload.doc_type)
    .bind(&payload.document_data)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify the supervisor for approval
    let supervisor_role = match user.role {
        Role::Physicist => "TheArtificer",
        Role::Chemist | Role::Biologist => "TheObserver",
        _ => "TheObserver",
    };

    let supervisor_ids: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(supervisor_role)
    .fetch_all(&state.db_pool)
    .await?;

    for (sup_id,) in &supervisor_ids {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'final_doc:submitted', $2::jsonb)
            "#,
        )
        .bind(sup_id)
        .bind(serde_json::json!({
            "document_id": doc_id.0,
            "doc_type": payload.doc_type,
            "experiment_id": payload.experiment_id,
            "submitted_by": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "final_object_documents",
        doc_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "type": payload.doc_type,
            "experiment_id": payload.experiment_id,
        })),
    )
    .await?;

    Ok(doc_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-PH-03/06, UC-CH-01/01A, UC-BIO-01/01A: View Archives
// ════════════════════════════════════════════════════════════════════════════════

/// Get science archive entries filtered by type (matter, physical_object, species).
///
/// **Access:** Any scientist or Administrator.
#[tauri::command]
pub async fn get_archive(
    state: State<'_, AppState>,
    archive_type: String,
) -> Result<Vec<ArchiveItem>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::Mathematician, Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    let valid_types = ["matter", "physical_object", "species"];
    if !valid_types.contains(&archive_type.as_str()) {
        return Err(AppError::Internal(
            "Archive type must be 'matter', 'physical_object', or 'species'.".into(),
        ));
    }

    // Check Redis cache first
    let cache_key = format!("science_archive:{}", archive_type);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(items) = serde_json::from_str::<Vec<ArchiveItem>>(&cached) {
                return Ok(items);
            }
        }
    }

    let items = sqlx::query_as::<_, ArchiveItem>(
        r#"
        SELECT id, type, name, classification, detail, storage_path, experiment_id, created_at
        FROM science_archive
        WHERE type = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(&archive_type)
    .fetch_all(&state.db_pool)
    .await?;

    // Cache for 1 hour
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&items) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 3600).await;
        }
    }

    Ok(items)
}

/// Get experiment archive filtered by type.
///
/// **Access:** Any scientist or Administrator.
#[tauri::command]
pub async fn get_experiment_archive(
    state: State<'_, AppState>,
    experiment_type: Option<String>,
) -> Result<Vec<ExperimentSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::Mathematician, Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    let experiments = if let Some(ref exp_type) = experiment_type {
        sqlx::query_as::<_, ExperimentSummary>(
            r#"
            SELECT e.id, e.title, e.type, e.status, e.proposed_by,
                   u.full_name AS proposer_name, e.metadata, e.created_at,
                   e.approved_at, e.closed_at
            FROM experiments e
            JOIN users u ON u.id = e.proposed_by
            WHERE e.type = $1 AND e.deleted_at IS NULL
            ORDER BY e.created_at DESC
            "#,
        )
        .bind(exp_type)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, ExperimentSummary>(
            r#"
            SELECT e.id, e.title, e.type, e.status, e.proposed_by,
                   u.full_name AS proposer_name, e.metadata, e.created_at,
                   e.approved_at, e.closed_at
            FROM experiments e
            JOIN users u ON u.id = e.proposed_by
            WHERE e.deleted_at IS NULL
            ORDER BY e.created_at DESC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(experiments)
}

/// Get experiment detail including logs.
///
/// **Access:** Any scientist or Administrator.
#[tauri::command]
pub async fn get_experiment_detail(
    state: State<'_, AppState>,
    experiment_id: Uuid,
) -> Result<(ExperimentSummary, Vec<ExperimentLogSummary>), AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::Mathematician, Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    let experiment = sqlx::query_as::<_, ExperimentSummary>(
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

    let logs = sqlx::query_as::<_, ExperimentLogSummary>(
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

/// Get math results for a task.
///
/// **Access:** Any scientist or Administrator.
#[tauri::command]
pub async fn get_math_results(
    state: State<'_, AppState>,
    task_id: Uuid,
) -> Result<Vec<MathResult>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::Mathematician, Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    let results = sqlx::query_as::<_, MathResult>(
        r#"
        SELECT id, task_id, submitted_by, content, pdf_storage_path, created_at
        FROM math_results
        WHERE task_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(task_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(results)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-PH-04 / UC-CH-02 / UC-BIO-05: View Approved Tests
// ════════════════════════════════════════════════════════════════════════════════

/// Get approved tests filtered by applicable scope.
///
/// **Access:** Any scientist or Administrator.
#[tauri::command]
pub async fn get_approved_tests(
    state: State<'_, AppState>,
    scope: Option<String>,
) -> Result<Vec<ApprovedTest>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Mathematician, Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    // Auto-determine scope from role if not provided
    let effective_scope = scope.or_else(|| {
        match user.role {
            Role::Physicist => Some("physical".to_string()),
            Role::Chemist => Some("matter".to_string()),
            Role::Biologist => Some("all_species".to_string()),
            _ => None,
        }
    });

    // Check Redis cache
    if let Some(ref s) = effective_scope {
        let cache_key = format!("approved_tests:{}", s);
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
                if let Ok(items) = serde_json::from_str::<Vec<ApprovedTest>>(&cached) {
                    return Ok(items);
                }
            }
        }
    }

    let tests = if let Some(ref s) = effective_scope {
        sqlx::query_as::<_, ApprovedTest>(
            r#"
            SELECT id, name, procedure, category, applicable_scope, accepted_at, created_at
            FROM approved_tests
            WHERE applicable_scope = $1 AND deleted_at IS NULL
            ORDER BY name
            "#,
        )
        .bind(s)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, ApprovedTest>(
            r#"
            SELECT id, name, procedure, category, applicable_scope, accepted_at, created_at
            FROM approved_tests
            WHERE deleted_at IS NULL
            ORDER BY name
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    };

    // Cache
    if let Some(ref s) = effective_scope {
        let cache_key = format!("approved_tests:{}", s);
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(json) = serde_json::to_string(&tests) {
                let _: Result<(), _> = conn.set_ex(&cache_key, &json, 3600).await;
            }
        }
    }

    Ok(tests)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-PH-05 / UC-CH-03 / UC-BIO-06: Propose New Test
// ════════════════════════════════════════════════════════════════════════════════

/// Propose a new test method. Routes to the supervisor Director for approval.
///
/// **Access:** Physicist, Chemist, Biologist, or Administrator.
#[tauri::command]
pub async fn propose_new_test(
    state: State<'_, AppState>,
    payload: ProposeNewTestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    if payload.name.trim().is_empty() {
        return Err(AppError::Internal("Test name cannot be empty.".into()));
    }

    let proposal_data = serde_json::json!({
        "name": payload.name,
        "goal": payload.goal,
        "procedure": payload.procedure,
        "species_scope": payload.species_scope,
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

    // Notify the supervisor director
    let supervisor_role = match user.role {
        Role::Physicist => "TheArtificer",
        Role::Chemist | Role::Biologist => "TheObserver",
        _ => "TheObserver",
    };

    let supervisor_ids: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(supervisor_role)
    .fetch_all(&state.db_pool)
    .await?;

    for (sup_id,) in &supervisor_ids {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'test_proposal:submitted', $2::jsonb)
            "#,
        )
        .bind(sup_id)
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
        Some(serde_json::json!({
            "name": payload.name,
        })),
    )
    .await?;

    Ok(prop_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-BIO-02 (species-specific): Propose New Species
// ════════════════════════════════════════════════════════════════════════════════

/// Propose a new species for documentation (treated as biology_observation experiment).
///
/// **Access:** Biologist or Administrator.
#[tauri::command]
pub async fn propose_new_species(
    state: State<'_, AppState>,
    payload: ProposeNewSpeciesPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth!(state, Role::Biologist);

    if payload.title.trim().is_empty() {
        return Err(AppError::Internal("Species proposal title cannot be empty.".into()));
    }

    let metadata = serde_json::json!({
        "introduction": payload.introduction,
        "problem_statement": payload.problem_statement,
        "research_questions": payload.research_questions,
        "hypotheses": payload.hypotheses,
        "location": payload.location,
        "species_specific": true,
    });

    let exp_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO experiments (title, type, metadata, proposed_by)
        VALUES ($1, 'biology_observation', $2, $3)
        RETURNING id
        "#,
    )
    .bind(&payload.title)
    .bind(&metadata)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Create request routed to The Observer
    let req_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, requester_id, title, description, payload, bypass_authority)
        VALUES ('experiment_proposal', $1, $2, $3, $4, 'TheObserver')
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.title)
    .bind(payload.introduction.as_deref())
    .bind(serde_json::json!({
        "experiment_id": exp_id.0,
        "experiment_type": "biology_observation",
        "species_proposal": true,
    }))
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
            VALUES ($1, 'species:proposed', $2::jsonb)
            "#,
        )
        .bind(obs_id)
        .bind(serde_json::json!({
            "request_id": req_id.0,
            "experiment_id": exp_id.0,
            "title": payload.title,
            "proposed_by": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "experiments",
        exp_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "title": payload.title,
            "type": "biology_observation",
            "species_proposal": true,
        })),
    )
    .await?;

    Ok(exp_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// Get Final Documents for an experiment
// ════════════════════════════════════════════════════════════════════════════════

/// Get final documents for an experiment.
///
/// **Access:** Any scientist or Administrator.
#[tauri::command]
pub async fn get_final_documents(
    state: State<'_, AppState>,
    experiment_id: Uuid,
) -> Result<Vec<FinalDocSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::Mathematician, Role::Physicist, Role::Chemist, Role::Biologist
    ]);

    let docs = sqlx::query_as::<_, FinalDocSummary>(
        r#"
        SELECT id, experiment_id, type, document_data, status, submitted_by, created_at
        FROM final_object_documents
        WHERE experiment_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(experiment_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(docs)
}

// ════════════════════════════════════════════════════════════════════════════════
// Get Test Proposals (for scientist to track their proposals)
// ════════════════════════════════════════════════════════════════════════════════

/// Get test proposals submitted by the current user.
///
/// **Access:** Any scientist or Administrator.
#[tauri::command]
pub async fn get_my_test_proposals(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::Physicist, Role::Chemist, Role::Biologist
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

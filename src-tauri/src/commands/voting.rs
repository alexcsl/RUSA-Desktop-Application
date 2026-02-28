// commands/voting.rs — Voting system commands (UC-DIR-01, UC-DIR-02, UC-DIR-04)
// Source of truth: 11_DIRECTORS.md, 00_MASTER_GUIDE.md §6
//
// Voting rules:
// - Sessions at 11:00 AM and 7:00 PM daily
// - Quorum: ≥ 8 of 13 Directors
// - Vote options: Yay, Nay, Abstain — each requires written reason
// - Directors may change vote while window is open
// - All Abstain = auto-denied
// - Sub-quorum: snoozed + 15-min push reminders
// - Administrator can override or terminate at any time
// - Redis key: vote_state:{session_id} — no TTL, delete on close

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

// ── Constants ─────────────────────────────────────────────────────────────────

const QUORUM_THRESHOLD: i64 = 8;

// ── Payloads & Response Structs ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CastVotePayload {
    pub session_id: Uuid,
    pub vote: String,   // 'yay', 'nay', 'abstain'
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct AdHocVotePayload {
    pub topic: String,
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminOverridePayload {
    pub session_id: Uuid,
    pub decision: String,  // 'approved' or 'denied'
    pub reason: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct VoteSessionSummary {
    pub id: Uuid,
    pub topic: String,
    pub context: Option<String>,
    pub status: String,
    pub opens_at: DateTime<Utc>,
    pub closes_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub total_yay: i32,
    pub total_nay: i32,
    pub total_abstain: i32,
    pub admin_overridden: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct VoteSessionDetail {
    pub id: Uuid,
    pub topic: String,
    pub context: Option<String>,
    pub status: String,
    pub opens_at: DateTime<Utc>,
    pub closes_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub total_yay: i32,
    pub total_nay: i32,
    pub total_abstain: i32,
    pub admin_overridden: bool,
    pub admin_override_decision: Option<String>,
    pub admin_override_reason: Option<String>,
    pub admin_terminated: bool,
    pub request_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct VoteRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub director_id: Uuid,
    pub vote: String,
    pub reason: String,
    pub changed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct VoteSessionWithRecords {
    pub session: VoteSessionDetail,
    pub records: Vec<VoteRecordWithName>,
    pub my_vote: Option<VoteRecord>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct VoteRecordWithName {
    pub id: Uuid,
    pub director_id: Uuid,
    pub director_name: String,
    pub vote: String,
    pub reason: String,
    pub changed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ── Helper: validate vote string ──────────────────────────────────────────────

fn validate_vote(vote: &str) -> Result<(), AppError> {
    match vote {
        "yay" | "nay" | "abstain" => Ok(()),
        _ => Err(AppError::Internal(
            "Vote must be 'yay', 'nay', or 'abstain'.".into(),
        )),
    }
}

/// Invalidate the Redis cache for a vote session.
async fn invalidate_vote_cache(state: &State<'_, AppState>, session_id: Uuid) {
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("vote_state:{}", session_id)).await;
    }
}

// ── UC-DIR-01: Cast Vote ──────────────────────────────────────────────────────

/// Cast a vote on an open voting session.
///
/// **Access:** Any Director or Administrator.
/// Rules: session must be 'open', director hasn't voted yet.
/// If this vote brings total to quorum, system tallies and closes.
#[tauri::command]
pub async fn cast_vote(
    state: State<'_, AppState>,
    payload: CastVotePayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_director!(state);
    validate_vote(&payload.vote)?;

    if payload.reason.trim().is_empty() {
        return Err(AppError::Internal("A written reason is required.".into()));
    }

    // 1. Verify session is open
    let session_status: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM vote_sessions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.session_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (status,) = session_status
        .ok_or_else(|| AppError::Internal("Vote session not found.".into()))?;

    if status != "open" && status != "quorum_pending" {
        return Err(AppError::Internal(
            "This voting session is no longer accepting votes.".into(),
        ));
    }

    // 2. Check director hasn't already voted
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM vote_records WHERE session_id = $1 AND director_id = $2",
    )
    .bind(payload.session_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Internal(
            "You have already cast a vote. Use change_vote to modify it.".into(),
        ));
    }

    // 3. Insert vote record
    let record_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_records (session_id, director_id, vote, reason)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(payload.session_id)
    .bind(user.id)
    .bind(&payload.vote)
    .bind(&payload.reason)
    .fetch_one(&state.db_pool)
    .await?;

    // 4. Update session tallies
    let tally_col = match payload.vote.as_str() {
        "yay" => "total_yay",
        "nay" => "total_nay",
        "abstain" => "total_abstain",
        _ => unreachable!(),
    };

    sqlx::query(&format!(
        "UPDATE vote_sessions SET {} = {} + 1, updated_at = NOW() WHERE id = $1",
        tally_col, tally_col
    ))
    .bind(payload.session_id)
    .execute(&state.db_pool)
    .await?;

    // 5. Check quorum
    let tallies: (i32, i32, i32) = sqlx::query_as(
        "SELECT total_yay, total_nay, total_abstain FROM vote_sessions WHERE id = $1",
    )
    .bind(payload.session_id)
    .fetch_one(&state.db_pool)
    .await?;

    let total_votes = (tallies.0 + tallies.1 + tallies.2) as i64;

    if total_votes >= QUORUM_THRESHOLD {
        // Quorum reached — determine result
        let result = if tallies.0 == 0 && tallies.1 == 0 {
            // All abstain → auto-denied
            "denied"
        } else if tallies.0 > tallies.1 {
            "approved"
        } else {
            "denied"
        };

        sqlx::query(
            r#"
            UPDATE vote_sessions
            SET status = 'decided', result = $2, closes_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(payload.session_id)
        .bind(result)
        .execute(&state.db_pool)
        .await?;

        // Update linked request status if exists
        sqlx::query(
            r#"
            UPDATE requests
            SET status = $2, updated_at = NOW()
            WHERE id = (SELECT request_id FROM vote_sessions WHERE id = $1 AND request_id IS NOT NULL)
            "#,
        )
        .bind(payload.session_id)
        .bind(result)
        .execute(&state.db_pool)
        .await?;
    }
    // If quorum not yet reached, status stays 'open' — no change needed.

    // 6. Audit log
    write_audit_log(
        &state.db_pool,
        "vote_records",
        record_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "session_id": payload.session_id,
            "director_id": user.id,
            "vote": payload.vote,
            "reason": payload.reason,
        })),
    )
    .await?;

    // 7. Invalidate Redis cache
    invalidate_vote_cache(&state, payload.session_id).await;

    Ok(())
}

// ── UC-DIR-01 (cont.): Change Vote ───────────────────────────────────────────

/// Change a previously cast vote while the session is still open.
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn change_vote(
    state: State<'_, AppState>,
    payload: CastVotePayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_director!(state);
    validate_vote(&payload.vote)?;

    if payload.reason.trim().is_empty() {
        return Err(AppError::Internal("A written reason is required.".into()));
    }

    // 1. Verify session is still open
    let session_status: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM vote_sessions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.session_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (status,) = session_status
        .ok_or_else(|| AppError::Internal("Vote session not found.".into()))?;

    if status != "open" && status != "quorum_pending" {
        return Err(AppError::Internal(
            "This voting session is no longer accepting vote changes.".into(),
        ));
    }

    // 2. Get existing vote for audit before_data
    let existing: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT id, vote, reason FROM vote_records WHERE session_id = $1 AND director_id = $2",
    )
    .bind(payload.session_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (record_id, old_vote, old_reason) = existing
        .ok_or_else(|| AppError::Internal("No existing vote to change.".into()))?;

    // 3. Update vote
    sqlx::query(
        r#"
        UPDATE vote_records
        SET vote = $1, reason = $2, changed_at = NOW()
        WHERE id = $3
        "#,
    )
    .bind(&payload.vote)
    .bind(&payload.reason)
    .bind(record_id)
    .execute(&state.db_pool)
    .await?;

    // 4. Recalculate session tallies (simplest: recount from records)
    sqlx::query(
        r#"
        UPDATE vote_sessions SET
          total_yay     = (SELECT COUNT(*) FROM vote_records WHERE session_id = $1 AND vote = 'yay'),
          total_nay     = (SELECT COUNT(*) FROM vote_records WHERE session_id = $1 AND vote = 'nay'),
          total_abstain = (SELECT COUNT(*) FROM vote_records WHERE session_id = $1 AND vote = 'abstain'),
          updated_at    = NOW()
        WHERE id = $1
        "#,
    )
    .bind(payload.session_id)
    .execute(&state.db_pool)
    .await?;

    // 5. Audit
    write_audit_log(
        &state.db_pool,
        "vote_records",
        record_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({
            "vote": old_vote,
            "reason": old_reason,
        })),
        Some(serde_json::json!({
            "vote": payload.vote,
            "reason": payload.reason,
        })),
    )
    .await?;

    // 6. Invalidate cache
    invalidate_vote_cache(&state, payload.session_id).await;

    Ok(())
}

// ── Get Pending Vote Sessions ─────────────────────────────────────────────────

/// List all open/quorum-pending vote sessions for the current Director.
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn get_pending_votes(
    state: State<'_, AppState>,
) -> Result<Vec<VoteSessionSummary>, AppError> {
    let _user = crate::require_auth_director!(state);

    let sessions = sqlx::query_as::<_, VoteSessionSummary>(
        r#"
        SELECT id, topic, context, status, opens_at, closes_at, result,
               total_yay, total_nay, total_abstain, admin_overridden,
               created_by, created_at
        FROM vote_sessions
        WHERE status IN ('open', 'quorum_pending')
          AND deleted_at IS NULL
        ORDER BY opens_at ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(sessions)
}

// ── Get Vote Session Detail ───────────────────────────────────────────────────

/// Get full detail for a specific vote session, including all vote records.
/// Vote records show director names only after the session is decided/closed.
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn get_vote_session_detail(
    state: State<'_, AppState>,
    session_id: Uuid,
) -> Result<VoteSessionWithRecords, AppError> {
    let user = crate::require_auth_director!(state);

    let session = sqlx::query_as::<_, VoteSessionDetail>(
        r#"
        SELECT id, topic, context, status, opens_at, closes_at, result,
               total_yay, total_nay, total_abstain,
               admin_overridden, admin_override_decision, admin_override_reason,
               admin_terminated, request_id, created_by, created_at
        FROM vote_sessions
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(session_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Vote session not found.".into()))?;

    // Records with director names (visible after session concludes, or to admin)
    let is_concluded = matches!(
        session.status.as_str(),
        "decided" | "overridden" | "terminated" | "closed"
    );

    let records = if is_concluded || user.role == Role::Administrator {
        sqlx::query_as::<_, VoteRecordWithName>(
            r#"
            SELECT vr.id, vr.director_id, u.full_name AS director_name,
                   vr.vote, vr.reason, vr.changed_at, vr.created_at
            FROM vote_records vr
            JOIN users u ON u.id = vr.director_id
            WHERE vr.session_id = $1
            ORDER BY vr.created_at ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        // While open: show anonymous aggregate only — no names
        Vec::new()
    };

    // Get current user's own vote (always visible to themselves)
    let my_vote = sqlx::query_as::<_, VoteRecord>(
        r#"
        SELECT id, session_id, director_id, vote, reason, changed_at, created_at
        FROM vote_records
        WHERE session_id = $1 AND director_id = $2
        "#,
    )
    .bind(session_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    Ok(VoteSessionWithRecords {
        session,
        records,
        my_vote,
    })
}

// ── Get All Vote Sessions (history) ───────────────────────────────────────────

/// Get all vote sessions (for archive/history view).
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn get_vote_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<VoteSessionSummary>, AppError> {
    let _user = crate::require_auth_director!(state);

    let sessions = sqlx::query_as::<_, VoteSessionSummary>(
        r#"
        SELECT id, topic, context, status, opens_at, closes_at, result,
               total_yay, total_nay, total_abstain, admin_overridden,
               created_by, created_at
        FROM vote_sessions
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(sessions)
}

// ── UC-DIR-04: Initiate Ad-Hoc Vote ──────────────────────────────────────────

/// Create a new ad-hoc voting session (not linked to a request).
///
/// **Access:** Any Director or Administrator.
/// The session follows standard quorum and timing rules.
#[tauri::command]
pub async fn initiate_ad_hoc_vote(
    state: State<'_, AppState>,
    payload: AdHocVotePayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_director!(state);

    if payload.topic.trim().is_empty() {
        return Err(AppError::Internal("Vote topic cannot be empty.".into()));
    }

    // Determine next voting window (11:00 or 19:00)
    let now = Utc::now();
    let opens_at = now; // For ad-hoc votes, open immediately

    let session_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (topic, context, status, opens_at, created_by)
        VALUES ($1, $2, 'open', $3, $4)
        RETURNING id
        "#,
    )
    .bind(&payload.topic)
    .bind(&payload.context)
    .bind(opens_at)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Audit
    write_audit_log(
        &state.db_pool,
        "vote_sessions",
        session_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "topic": payload.topic,
            "context": payload.context,
            "type": "ad_hoc",
            "opens_at": opens_at,
        })),
    )
    .await?;

    // Cache the vote state in Redis (no TTL — deleted on close)
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let cache_key = format!("vote_state:{}", session_id.0);
        let _: Result<(), _> = conn
            .set::<_, _, ()>(
                &cache_key,
                serde_json::json!({
                    "id": session_id.0,
                    "status": "open",
                    "total_yay": 0,
                    "total_nay": 0,
                    "total_abstain": 0,
                })
                .to_string(),
            )
            .await;
    }

    // Create notification for all directors
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'vote:new', $2::jsonb
        FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name IN (
            'GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
            'TheNomad','TheArtificer','TheObserver','TheWanderer',
            'TheTaskmaster','TheGuardian','TheStatistician','TheCoordinator',
            'TheOverseer','TheAnchorman','Administrator'
        )
        AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(session_id.0)
    .bind(serde_json::json!({
        "session_id": session_id.0,
        "topic": payload.topic,
    }))
    .execute(&state.db_pool)
    .await?;

    Ok(session_id.0)
}

// ── Administrator: Override Vote ──────────────────────────────────────────────

/// Administrator overrides a vote session with a direct decision.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_override_vote(
    state: State<'_, AppState>,
    payload: AdminOverridePayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_admin!(state);

    if payload.decision != "approved" && payload.decision != "denied" {
        return Err(AppError::Internal(
            "Decision must be 'approved' or 'denied'.".into(),
        ));
    }

    // Get before data for audit
    let before: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT status, result FROM vote_sessions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.session_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (old_status, old_result) = before
        .ok_or_else(|| AppError::Internal("Vote session not found.".into()))?;

    // Apply override
    sqlx::query(
        r#"
        UPDATE vote_sessions SET
          status = 'overridden',
          result = $2,
          admin_overridden = TRUE,
          admin_override_decision = $2,
          admin_override_reason = $3,
          closes_at = NOW(),
          updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(payload.session_id)
    .bind(&payload.decision)
    .bind(&payload.reason)
    .execute(&state.db_pool)
    .await?;

    // Update linked request if exists
    sqlx::query(
        r#"
        UPDATE requests SET status = $2, decided_by = $3, decision_reason = $4, updated_at = NOW()
        WHERE id = (SELECT request_id FROM vote_sessions WHERE id = $1 AND request_id IS NOT NULL)
        "#,
    )
    .bind(payload.session_id)
    .bind(&payload.decision)
    .bind(user.id)
    .bind(&payload.reason)
    .execute(&state.db_pool)
    .await?;

    // Audit
    write_audit_log(
        &state.db_pool,
        "vote_sessions",
        payload.session_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({
            "status": old_status,
            "result": old_result,
        })),
        Some(serde_json::json!({
            "status": "overridden",
            "result": payload.decision,
            "admin_override_reason": payload.reason,
        })),
    )
    .await?;

    // Invalidate cache
    invalidate_vote_cache(&state, payload.session_id).await;

    Ok(())
}

// ── Administrator: Terminate Vote ─────────────────────────────────────────────

/// Administrator terminates a vote session without deciding.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_terminate_vote(
    state: State<'_, AppState>,
    session_id: Uuid,
    reason: String,
) -> Result<(), AppError> {
    let user = crate::require_auth_admin!(state);

    let before: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM vote_sessions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(session_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (old_status,) = before
        .ok_or_else(|| AppError::Internal("Vote session not found.".into()))?;

    sqlx::query(
        r#"
        UPDATE vote_sessions SET
          status = 'terminated',
          admin_terminated = TRUE,
          admin_override_reason = $2,
          closes_at = NOW(),
          updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(session_id)
    .bind(&reason)
    .execute(&state.db_pool)
    .await?;

    // Update request status to cancelled
    sqlx::query(
        r#"
        UPDATE requests SET status = 'cancelled', updated_at = NOW()
        WHERE id = (SELECT request_id FROM vote_sessions WHERE id = $1 AND request_id IS NOT NULL)
        "#,
    )
    .bind(session_id)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "vote_sessions",
        session_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": old_status })),
        Some(serde_json::json!({
            "status": "terminated",
            "admin_terminated": true,
            "reason": reason,
        })),
    )
    .await?;

    invalidate_vote_cache(&state, session_id).await;

    Ok(())
}

// reminders.rs — Scheduled background tasks for voting reminders
//
// Design Pattern: Observer — this task observes the state of open vote sessions
// and triggers push notifications when quorum has not yet been reached.
//
// Rule (00_MASTER_GUIDE.md §6):
//   "Sub-quorum: Vote is snoozed and 15-minute push reminders are sent
//    until quorum is reached."
//
// Behavior:
//   Every 15 minutes, find all vote sessions that are 'open' or 'quorum_pending'
//   AND whose voting window has started (opens_at <= NOW()) AND quorum < 8.
//   For each session, insert a 'vote:reminder' notification for every Director
//   who has NOT yet cast a vote.

use std::time::Duration;
use sqlx::PgPool;
use tauri::Emitter;
use uuid::Uuid;

const QUORUM_THRESHOLD: i64 = 8;
const REMINDER_INTERVAL_SECS: u64 = 60 * 15; // 15 minutes

pub async fn run_voting_reminders(app_handle: tauri::AppHandle, db_pool: PgPool) {
    loop {
        tokio::time::sleep(Duration::from_secs(REMINDER_INTERVAL_SECS)).await;

        if let Err(e) = send_subquorum_reminders(&app_handle, &db_pool).await {
            eprintln!("[reminders] Sub-quorum reminder error: {}", e);
        }
    }
}

/// Find every open vote session below quorum and notify directors who haven't voted.
async fn send_subquorum_reminders(
    app_handle: &tauri::AppHandle,
    db_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    // Fetch all open/quorum_pending sessions whose window is active and
    // that still haven't reached quorum.
    let sessions: Vec<(Uuid, String, i64)> = sqlx::query_as(
        r#"
        SELECT
            vs.id,
            vs.topic,
            COUNT(vr.id) AS vote_count
        FROM vote_sessions vs
        LEFT JOIN vote_records vr ON vr.session_id = vs.id
        WHERE vs.status IN ('open', 'quorum_pending')
          AND vs.opens_at <= NOW()
          AND vs.deleted_at IS NULL
        GROUP BY vs.id, vs.topic
        HAVING COUNT(vr.id) < $1
        "#,
    )
    .bind(QUORUM_THRESHOLD)
    .fetch_all(db_pool)
    .await?;

    for (session_id, topic, vote_count) in sessions {
        // Find directors who have NOT yet voted in this session.
        let unsent: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT u.id
            FROM users u
            JOIN roles r ON r.id = u.role_id
            WHERE r.name IN (
                'GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
                'TheNomad','TheArtificer','TheObserver','TheWanderer',
                'TheTaskmaster','TheGuardian','TheStatistician','TheCoordinator',
                'TheOverseer','TheAnchorman'
            )
              AND u.deleted_at IS NULL
              AND u.is_active = TRUE
              AND u.id NOT IN (
                  SELECT director_id FROM vote_records WHERE session_id = $1
              )
            "#,
        )
        .bind(session_id)
        .fetch_all(db_pool)
        .await?;

        if unsent.is_empty() {
            continue;
        }

        let payload = serde_json::json!({
            "session_id": session_id,
            "topic": topic,
            "votes_cast": vote_count,
            "quorum_needed": QUORUM_THRESHOLD,
        });

        // Insert a reminder notification for each director who hasn't voted.
        for (uid,) in &unsent {
            sqlx::query(
                "INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'vote:reminder', $2)",
            )
            .bind(uid)
            .bind(&payload)
            .execute(db_pool)
            .await?;
        }

        // Emit Tauri event so any open Directors window reacts immediately.
        let _ = app_handle.emit("vote:updated", &payload);
    }

    Ok(())
}

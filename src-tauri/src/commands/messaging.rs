// commands/messaging.rs — Universal Messaging System
// Source of truth: 00_MASTER_GUIDE.md §6 (Messaging System),
//   03_SECURITY_TEAMS.md (UC-SH-04, UC-SS-02),
//   09_MEDICAL.md (UC-HOM-04),
//   11_DIRECTORS.md (UC-GUA-04, UC-OVR-02, UC-ANC-02),
//   Internal_Docs.md (Secure Global Communications)
//
// This file implements the shared messaging infrastructure used across
// all channels: general, security, medical_heads, broadcast.
//
// Channels & access matrix:
// ┌─────────────────┬─────────────────────────────────────────────────────┬────────────────────┐
// │ Channel         │ Can Send                                            │ Can Read           │
// ├─────────────────┼─────────────────────────────────────────────────────┼────────────────────┤
// │ general         │ All authenticated users                             │ All authenticated  │
// │ security        │ GalacticSecurityHead, GalacticSecurityStaff,        │ + TheOverseer      │
// │                 │ TheGuardian, Administrator                          │   (read-only)      │
// │ medical_heads   │ HeadOfMedicine, Administrator                       │ Same as writers    │
// │ broadcast       │ TheAnchorman, TheGuardian, Administrator            │ All (receive)      │
// └─────────────────┴─────────────────────────────────────────────────────┴────────────────────┘
//
// Features: To/CC/BCC, scheduled send, message recall (if unread),
//           group targets, file attachments, real-time push delivery.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::{
    audit::{write_audit_log, AuditOperation},
    error::AppError,
    state::{AppState, AuthenticatedUser, Role},
};

// ══════════════════════════════════════════════════════════════════════════════
//  Channel access helpers
// ══════════════════════════════════════════════════════════════════════════════

/// Roles that can SEND on the security channel.
const SECURITY_SEND_ROLES: &[Role] = &[
    Role::GalacticSecurityHead,
    Role::GalacticSecurityStaff,
    Role::TheGuardian,
];

/// Roles that can READ the security channel (includes Overseer read-only).
const SECURITY_READ_ROLES: &[Role] = &[
    Role::GalacticSecurityHead,
    Role::GalacticSecurityStaff,
    Role::TheGuardian,
    Role::TheOverseer,
];

/// Roles that can send on the medical_heads channel.
const MEDICAL_HEADS_SEND_ROLES: &[Role] = &[Role::HeadOfMedicine];

/// Roles that can send on the broadcast channel.
const BROADCAST_SEND_ROLES: &[Role] = &[
    Role::TheAnchorman,
    Role::TheGuardian,
];

/// Check if the user can SEND on the given channel.
fn can_send_on_channel(user: &AuthenticatedUser, channel: &str) -> bool {
    if user.role == Role::Administrator {
        return true;
    }
    match channel {
        "general" => true, // all authenticated users
        "security" => SECURITY_SEND_ROLES.contains(&user.role),
        "medical_heads" => MEDICAL_HEADS_SEND_ROLES.contains(&user.role),
        "broadcast" => BROADCAST_SEND_ROLES.contains(&user.role),
        _ => false,
    }
}

/// Check if the user can READ the given channel.
fn can_read_channel(user: &AuthenticatedUser, channel: &str) -> bool {
    if user.role == Role::Administrator {
        return true;
    }
    match channel {
        "general" => true,
        "security" => SECURITY_READ_ROLES.contains(&user.role),
        "medical_heads" => MEDICAL_HEADS_SEND_ROLES.contains(&user.role),
        "broadcast" => true, // everyone receives broadcasts
        _ => false,
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Payloads & Response Types
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct SendMessagePayload {
    pub subject: String,
    pub body: String,
    pub channel: String,
    pub recipients_to: Vec<Uuid>,
    pub recipients_cc: Option<Vec<Uuid>>,
    pub recipients_bcc: Option<Vec<Uuid>>,
    /// If set, message is not delivered until this time.
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Group IDs — members of these groups are auto-added as 'to' recipients.
    pub group_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MessageSummary {
    pub id: Uuid,
    pub from_id: Uuid,
    pub from_name: String,
    pub subject: String,
    pub body: String,
    pub channel: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub recalled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MessageRecipientRow {
    pub user_id: Uuid,
    pub full_name: String,
    #[sqlx(rename = "type")]
    pub recipient_type: String,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct MessageDetail {
    pub message: MessageSummary,
    pub recipients: Vec<MessageRecipientRow>,
    pub attachments: Vec<AttachmentSummary>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AttachmentSummary {
    pub id: Uuid,
    pub original_filename: String,
    pub content_type: String,
    pub storage_path: String,
    pub file_size_bytes: Option<i64>,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct InboxMessage {
    pub id: Uuid,
    pub from_id: Uuid,
    pub from_name: String,
    pub subject: String,
    pub channel: String,
    pub recipient_type: String,
    pub read_at: Option<DateTime<Utc>>,
    pub recalled_at: Option<DateTime<Utc>>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GroupSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GroupMemberRow {
    pub user_id: Uuid,
    pub full_name: String,
    pub role_name: String,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EligibleRecipient {
    pub id: Uuid,
    pub full_name: String,
    pub username: String,
    pub role_name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UnreadCount {
    pub channel: String,
    pub count: i64,
}

// ══════════════════════════════════════════════════════════════════════════════
//  COMMANDS
// ══════════════════════════════════════════════════════════════════════════════

// ── Send Message ──────────────────────────────────────────────────────────────

/// Send a message on any channel.
/// Validates channel access, inserts message + recipients, writes audit log.
/// If group_ids are provided, their members are expanded into 'to' recipients.
///
/// **Access:** Channel-specific (see access matrix above).
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    payload: SendMessagePayload,
) -> Result<Uuid, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    // Validate channel value
    let valid_channels = ["general", "security", "medical_heads", "broadcast"];
    if !valid_channels.contains(&payload.channel.as_str()) {
        return Err(AppError::Internal(format!(
            "Invalid channel '{}'. Must be one of: {}",
            payload.channel,
            valid_channels.join(", ")
        )));
    }

    // Channel send authorization
    if !can_send_on_channel(&user, &payload.channel) {
        return Err(AppError::Forbidden);
    }

    // Validate at least one recipient
    if payload.recipients_to.is_empty() && payload.group_ids.as_ref().map_or(true, |g| g.is_empty()) {
        return Err(AppError::Internal(
            "At least one 'To' recipient or group is required.".into(),
        ));
    }

    if payload.subject.trim().is_empty() || payload.body.trim().is_empty() {
        return Err(AppError::Internal(
            "Subject and body are required.".into(),
        ));
    }

    // Insert the message
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO messages (from_id, subject, body, channel, scheduled_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.subject)
    .bind(&payload.body)
    .bind(&payload.channel)
    .bind(payload.scheduled_at)
    .fetch_one(&state.db_pool)
    .await?;

    let msg_id = row.0;

    // Expand group members into 'to' recipients
    let mut all_to = payload.recipients_to.clone();
    if let Some(group_ids) = &payload.group_ids {
        for gid in group_ids {
            let members: Vec<(Uuid,)> = sqlx::query_as(
                "SELECT user_id FROM group_members WHERE group_id = $1 AND removed_at IS NULL",
            )
            .bind(gid)
            .fetch_all(&state.db_pool)
            .await?;
            for (uid,) in members {
                if !all_to.contains(&uid) {
                    all_to.push(uid);
                }
            }
        }
    }

    // Insert TO recipients (batch)
    for uid in &all_to {
        sqlx::query(
            "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'to') ON CONFLICT DO NOTHING",
        )
        .bind(msg_id)
        .bind(uid)
        .execute(&state.db_pool)
        .await?;
    }

    // Insert CC recipients
    if let Some(cc) = &payload.recipients_cc {
        for uid in cc {
            sqlx::query(
                "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'cc') ON CONFLICT DO NOTHING",
            )
            .bind(msg_id)
            .bind(uid)
            .execute(&state.db_pool)
            .await?;
        }
    }

    // Insert BCC recipients
    if let Some(bcc) = &payload.recipients_bcc {
        for uid in bcc {
            sqlx::query(
                "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'bcc') ON CONFLICT DO NOTHING",
            )
            .bind(msg_id)
            .bind(uid)
            .execute(&state.db_pool)
            .await?;
        }
    }

    // Write audit log
    write_audit_log(
        &state.db_pool,
        "messages",
        msg_id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "subject": payload.subject,
            "channel": payload.channel,
            "to_count": all_to.len(),
        })),
    )
    .await?;

    // Insert notification for each recipient (except sender)
    {
        let preview = if payload.body.len() > 100 {
            format!("{}…", &payload.body[..100])
        } else {
            payload.body.clone()
        };

        let all_recipient_ids = collect_all_recipient_ids(&all_to, &payload.recipients_cc, &payload.recipients_bcc);
        for uid in all_recipient_ids {
            if uid == user.id {
                continue;
            }
            sqlx::query(
                r#"
                INSERT INTO notifications (user_id, type, payload)
                VALUES ($1, 'message:new', $2)
                "#,
            )
            .bind(uid)
            .bind(serde_json::json!({
                "message_id": msg_id,
                "channel": payload.channel,
                "from_name": user.full_name,
                "subject": payload.subject,
                "preview": preview,
            }))
            .execute(&state.db_pool)
            .await?;
        }

        // Invalidate unread count cache for ALL affected recipients (to + cc + bcc)
        let all_invalidate = collect_all_recipient_ids(&all_to, &payload.recipients_cc, &payload.recipients_bcc);
        let mut redis_conn = state.redis_client.get_multiplexed_async_connection().await
            .map_err(|e| AppError::Cache(e.to_string()))?;
        for uid in &all_invalidate {
            let _: Result<(), _> = redis::AsyncCommands::del(
                &mut redis_conn,
                format!("unread_messages:{}", uid),
            ).await;
        }
    }

    Ok(msg_id)
}

/// Collect all unique recipient UUIDs from to/cc/bcc lists.
fn collect_all_recipient_ids(
    to: &[Uuid],
    cc: &Option<Vec<Uuid>>,
    bcc: &Option<Vec<Uuid>>,
) -> Vec<Uuid> {
    let mut all = to.to_vec();
    if let Some(cc_list) = cc {
        for uid in cc_list {
            if !all.contains(uid) {
                all.push(*uid);
            }
        }
    }
    if let Some(bcc_list) = bcc {
        for uid in bcc_list {
            if !all.contains(uid) {
                all.push(*uid);
            }
        }
    }
    all
}

// ── Get Inbox ─────────────────────────────────────────────────────────────────

/// Retrieve inbox messages for the current user on a specific channel.
/// Returns messages where the user is a recipient (to/cc/bcc).
/// For monitoring roles (TheOverseer on security), returns ALL channel messages.
///
/// **Access:** Channel-specific read access.
#[tauri::command]
pub async fn get_inbox(
    state: State<'_, AppState>,
    channel: String,
) -> Result<Vec<InboxMessage>, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    if !can_read_channel(&user, &channel) {
        return Err(AppError::Forbidden);
    }

    // Overseer monitoring mode: see ALL messages on the security channel
    let is_monitor = channel == "security" && user.role == Role::TheOverseer;

    let messages = if is_monitor {
        sqlx::query_as::<_, InboxMessage>(
            r#"
            SELECT m.id, m.from_id, u.full_name AS from_name, m.subject,
                   m.channel, COALESCE(mr.type, 'monitor') AS recipient_type,
                   mr.read_at,
                   m.recalled_at, m.scheduled_at, m.created_at
            FROM messages m
            JOIN users u ON u.id = m.from_id
            LEFT JOIN message_recipients mr ON mr.message_id = m.id AND mr.user_id = $1
            WHERE m.channel = $2
              AND m.deleted_at IS NULL
            ORDER BY m.created_at DESC
            LIMIT 200
            "#,
        )
        .bind(user.id)
        .bind(&channel)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, InboxMessage>(
            r#"
            SELECT m.id, m.from_id, u.full_name AS from_name, m.subject,
                   m.channel, mr.type AS recipient_type, mr.read_at,
                   m.recalled_at, m.scheduled_at, m.created_at
            FROM messages m
            JOIN users u ON u.id = m.from_id
            JOIN message_recipients mr ON mr.message_id = m.id AND mr.user_id = $1
            WHERE m.channel = $2
              AND m.deleted_at IS NULL
            ORDER BY m.created_at DESC
            LIMIT 200
            "#,
        )
        .bind(user.id)
        .bind(&channel)
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(messages)
}

// ── Get Sent Messages ─────────────────────────────────────────────────────────

/// Retrieve messages sent by the current user on a specific channel.
///
/// **Access:** Channel-specific send access.
#[tauri::command]
pub async fn get_sent_messages(
    state: State<'_, AppState>,
    channel: String,
) -> Result<Vec<MessageSummary>, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    if !can_send_on_channel(&user, &channel) {
        return Err(AppError::Forbidden);
    }

    let messages = sqlx::query_as::<_, MessageSummary>(
        r#"
        SELECT m.id, m.from_id, u.full_name AS from_name, m.subject,
               m.body, m.channel, m.scheduled_at, m.recalled_at, m.created_at
        FROM messages m
        JOIN users u ON u.id = m.from_id
        WHERE m.from_id = $1
          AND m.channel = $2
          AND m.deleted_at IS NULL
        ORDER BY m.created_at DESC
        LIMIT 200
        "#,
    )
    .bind(user.id)
    .bind(&channel)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(messages)
}

// ── Get Message Detail ────────────────────────────────────────────────────────

/// Retrieve full message detail including recipients and attachments.
///
/// **Access:** Must be sender or recipient of the message, with channel read access.
#[tauri::command]
pub async fn get_message_detail(
    state: State<'_, AppState>,
    message_id: Uuid,
) -> Result<MessageDetail, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    // Fetch message
    let message = sqlx::query_as::<_, MessageSummary>(
        r#"
        SELECT m.id, m.from_id, u.full_name AS from_name, m.subject,
               m.body, m.channel, m.scheduled_at, m.recalled_at, m.created_at
        FROM messages m
        JOIN users u ON u.id = m.from_id
        WHERE m.id = $1 AND m.deleted_at IS NULL
        "#,
    )
    .bind(message_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::Internal("Message not found.".into()))?;

    // Verify channel read access
    if !can_read_channel(&user, &message.channel) {
        return Err(AppError::Forbidden);
    }

    // Verify user is sender or recipient (Overseer can monitor security channel)
    let is_sender = message.from_id == user.id;
    let is_monitor = message.channel == "security" && user.role == Role::TheOverseer;
    if !is_sender && !is_monitor {
        let is_recipient: Option<(Uuid,)> = sqlx::query_as(
            "SELECT user_id FROM message_recipients WHERE message_id = $1 AND user_id = $2",
        )
        .bind(message_id)
        .bind(user.id)
        .fetch_optional(&state.db_pool)
        .await?;

        if is_recipient.is_none() && user.role != Role::Administrator {
            return Err(AppError::Forbidden);
        }
    }

    // Fetch recipients (exclude BCC for non-sender, non-admin)
    let recipients = if is_sender || user.role == Role::Administrator {
        sqlx::query_as::<_, MessageRecipientRow>(
            r#"
            SELECT mr.user_id, u.full_name, mr.type, mr.read_at
            FROM message_recipients mr
            JOIN users u ON u.id = mr.user_id
            WHERE mr.message_id = $1
            ORDER BY mr.type ASC, u.full_name ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        // Non-sender: hide BCC recipients
        sqlx::query_as::<_, MessageRecipientRow>(
            r#"
            SELECT mr.user_id, u.full_name, mr.type, mr.read_at
            FROM message_recipients mr
            JOIN users u ON u.id = mr.user_id
            WHERE mr.message_id = $1 AND mr.type != 'bcc'
            ORDER BY mr.type ASC, u.full_name ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&state.db_pool)
        .await?
    };

    // Fetch attachments
    let attachments = sqlx::query_as::<_, AttachmentSummary>(
        r#"
        SELECT id, original_filename, content_type, storage_path,
               file_size_bytes, uploaded_at
        FROM message_attachments
        WHERE message_id = $1 AND deleted_at IS NULL
        ORDER BY uploaded_at ASC
        "#,
    )
    .bind(message_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(MessageDetail {
        message,
        recipients,
        attachments,
    })
}

// ── Mark Message Read ─────────────────────────────────────────────────────────

/// Mark a message as read for the current user.
///
/// **Access:** Any authenticated user who is a recipient.
#[tauri::command]
pub async fn msg_mark_read(
    state: State<'_, AppState>,
    message_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    sqlx::query(
        "UPDATE message_recipients SET read_at = NOW() WHERE message_id = $1 AND user_id = $2 AND read_at IS NULL",
    )
    .bind(message_id)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    // Invalidate unread cache
    let mut redis_conn = state.redis_client.get_multiplexed_async_connection().await
        .map_err(|e| AppError::Cache(e.to_string()))?;
    let _: Result<(), _> = redis::AsyncCommands::del(
        &mut redis_conn,
        format!("unread_messages:{}", user.id),
    ).await;

    Ok(())
}

// ── Recall Message ────────────────────────────────────────────────────────────

/// Recall (unsend) a message — only if no recipient has read it yet.
///
/// **Access:** Only the message sender. Channel send access required.
#[tauri::command]
pub async fn msg_recall(
    state: State<'_, AppState>,
    message_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    // Verify sender owns this message
    let msg: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT from_id, channel FROM messages WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(message_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (from_id, channel) = msg.ok_or(AppError::Internal("Message not found.".into()))?;
    if from_id != user.id && user.role != Role::Administrator {
        return Err(AppError::Internal(
            "Only the sender can recall a message.".into(),
        ));
    }

    // Channel send access check
    if !can_send_on_channel(&user, &channel) {
        return Err(AppError::Forbidden);
    }

    // Check if any recipient has read it
    let read_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM message_recipients WHERE message_id = $1 AND read_at IS NOT NULL",
    )
    .bind(message_id)
    .fetch_one(&state.db_pool)
    .await?;

    if read_count.0 > 0 {
        return Err(AppError::Internal(
            "Cannot recall — at least one recipient has already read it.".into(),
        ));
    }

    sqlx::query("UPDATE messages SET recalled_at = NOW() WHERE id = $1")
        .bind(message_id)
        .execute(&state.db_pool)
        .await?;

    write_audit_log(
        &state.db_pool,
        "messages",
        message_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({ "action": "recall" })),
    )
    .await?;

    Ok(())
}

// ── Soft-Delete Message ───────────────────────────────────────────────────────

/// Soft-delete a message (sender only). Never hard-deletes.
///
/// **Access:** Only the message sender, or Administrator.
#[tauri::command]
pub async fn delete_message(
    state: State<'_, AppState>,
    message_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    let msg: Option<(Uuid,)> = sqlx::query_as(
        "SELECT from_id FROM messages WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(message_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (from_id,) = msg.ok_or(AppError::Internal("Message not found.".into()))?;
    if from_id != user.id && user.role != Role::Administrator {
        return Err(AppError::Forbidden);
    }

    sqlx::query(
        "UPDATE messages SET deleted_at = NOW(), deleted_by = $1 WHERE id = $2",
    )
    .bind(user.id)
    .bind(message_id)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "messages",
        message_id,
        AuditOperation::Delete,
        user.id,
        None,
        None,
    )
    .await?;

    Ok(())
}

// ── Unread Count ──────────────────────────────────────────────────────────────

/// Get unread message counts per channel for the current user.
/// Results are cached in Redis for 5 minutes.
///
/// **Access:** Any authenticated user.
#[tauri::command]
pub async fn get_unread_counts(
    state: State<'_, AppState>,
) -> Result<Vec<UnreadCount>, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    // Try Redis cache first
    let cache_key = format!("unread_messages:{}", user.id);
    let mut redis_conn = state.redis_client.get_multiplexed_async_connection().await
        .map_err(|e| AppError::Cache(e.to_string()))?;

    let cached: Option<String> = redis::AsyncCommands::get(&mut redis_conn, &cache_key)
        .await
        .unwrap_or(None);

    if let Some(cached_json) = cached {
        if let Ok(counts) = serde_json::from_str::<Vec<UnreadCount>>(&cached_json) {
            return Ok(counts);
        }
    }

    // Cache miss — query DB
    let counts = sqlx::query_as::<_, UnreadCount>(
        r#"
        SELECT m.channel, COUNT(*)::BIGINT AS count
        FROM message_recipients mr
        JOIN messages m ON m.id = mr.message_id
        WHERE mr.user_id = $1
          AND mr.read_at IS NULL
          AND m.deleted_at IS NULL
          AND m.recalled_at IS NULL
        GROUP BY m.channel
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    // Cache for 5 minutes
    if let Ok(json) = serde_json::to_string(&counts) {
        let _: Result<(), _> = redis::AsyncCommands::set_ex(
            &mut redis_conn,
            &cache_key,
            json,
            300, // 5 minutes
        )
        .await;
    }

    Ok(counts)
}

// ══════════════════════════════════════════════════════════════════════════════
//  Eligible Recipients (for compose form pickers)
// ══════════════════════════════════════════════════════════════════════════════

/// Get eligible recipients for a channel.
/// - general: all active users
/// - security: security heads, staff, Guardian, Overseer
/// - medical_heads: all HeadOfMedicine users
/// - broadcast: all active users (broadcast targets)
///
/// **Access:** Must have send access on the channel.
#[tauri::command]
pub async fn get_eligible_recipients(
    state: State<'_, AppState>,
    channel: String,
) -> Result<Vec<EligibleRecipient>, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    if !can_send_on_channel(&user, &channel) {
        return Err(AppError::Forbidden);
    }

    let role_filter: Option<Vec<&str>> = match channel.as_str() {
        "security" => Some(vec![
            "GalacticSecurityHead",
            "GalacticSecurityStaff",
            "TheGuardian",
            "TheOverseer",
            "Administrator",
        ]),
        "medical_heads" => Some(vec!["HeadOfMedicine", "Administrator"]),
        _ => None, // general + broadcast → all active users
    };

    let recipients = if let Some(roles) = role_filter {
        sqlx::query_as::<_, EligibleRecipient>(
            r#"
            SELECT u.id, u.full_name, u.username, r.name AS role_name
            FROM users u
            JOIN roles r ON r.id = u.role_id
            WHERE u.deleted_at IS NULL AND u.is_active = TRUE
              AND r.name = ANY($1)
            ORDER BY u.full_name ASC
            "#,
        )
        .bind(&roles)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, EligibleRecipient>(
            r#"
            SELECT u.id, u.full_name, u.username, r.name AS role_name
            FROM users u
            JOIN roles r ON r.id = u.role_id
            WHERE u.deleted_at IS NULL AND u.is_active = TRUE
            ORDER BY u.full_name ASC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(recipients)
}

// ══════════════════════════════════════════════════════════════════════════════
//  GROUP MANAGEMENT
// ══════════════════════════════════════════════════════════════════════════════

/// Create a messaging group with initial members.
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn create_messaging_group(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_director!(state);

    if name.trim().is_empty() {
        return Err(AppError::Internal("Group name is required.".into()));
    }

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO groups (name, description, created_by)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    let group_id = row.0;

    write_audit_log(
        &state.db_pool,
        "groups",
        group_id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "name": name,
        })),
    )
    .await?;

    Ok(group_id)
}

/// List all messaging groups.
///
/// **Access:** Any authenticated user.
#[tauri::command]
pub async fn get_messaging_groups(
    state: State<'_, AppState>,
) -> Result<Vec<GroupSummary>, AppError> {
    let _user = crate::auth::get_current_user(&state).await?;

    let groups = sqlx::query_as::<_, GroupSummary>(
        r#"
        SELECT g.id, g.name, g.description,
               (SELECT COUNT(*) FROM group_members gm WHERE gm.group_id = g.id AND gm.removed_at IS NULL) AS member_count,
               g.created_by, g.created_at
        FROM groups g
        WHERE g.deleted_at IS NULL
        ORDER BY g.name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(groups)
}

/// Get members of a messaging group.
///
/// **Access:** Any authenticated user.
#[tauri::command]
pub async fn get_group_members(
    state: State<'_, AppState>,
    group_id: Uuid,
) -> Result<Vec<GroupMemberRow>, AppError> {
    let _user = crate::auth::get_current_user(&state).await?;

    let members = sqlx::query_as::<_, GroupMemberRow>(
        r#"
        SELECT gm.user_id, u.full_name, r.name AS role_name, gm.added_at
        FROM group_members gm
        JOIN users u ON u.id = gm.user_id
        JOIN roles r ON r.id = u.role_id
        WHERE gm.group_id = $1 AND gm.removed_at IS NULL
        ORDER BY u.full_name ASC
        "#,
    )
    .bind(group_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(members)
}

/// Add a member to a messaging group.
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn add_group_member(
    state: State<'_, AppState>,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let actor = crate::require_auth_director!(state);

    sqlx::query(
        r#"
        INSERT INTO group_members (group_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (group_id, user_id) DO UPDATE SET removed_at = NULL
        "#,
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "group_members",
        group_id,
        AuditOperation::Create,
        actor.id,
        None,
        Some(serde_json::json!({ "user_id": user_id })),
    )
    .await?;

    Ok(())
}

/// Remove a member from a messaging group (soft-remove).
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn remove_group_member(
    state: State<'_, AppState>,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let actor = crate::require_auth_director!(state);

    sqlx::query(
        "UPDATE group_members SET removed_at = NOW() WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "group_members",
        group_id,
        AuditOperation::Delete,
        actor.id,
        Some(serde_json::json!({ "user_id": user_id })),
        None,
    )
    .await?;

    Ok(())
}

/// Soft-delete a messaging group.
///
/// **Access:** Group creator (any Director), or Administrator.
#[tauri::command]
pub async fn delete_messaging_group(
    state: State<'_, AppState>,
    group_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::require_auth_director!(state);

    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT created_by FROM groups WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(group_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (created_by,) = row.ok_or(AppError::Internal("Group not found.".into()))?;
    if created_by != user.id && user.role != Role::Administrator {
        return Err(AppError::Forbidden);
    }

    sqlx::query(
        "UPDATE groups SET deleted_at = NOW(), deleted_by = $1 WHERE id = $2",
    )
    .bind(user.id)
    .bind(group_id)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "groups",
        group_id,
        AuditOperation::Delete,
        user.id,
        Some(serde_json::json!({ "group_id": group_id })),
        None,
    )
    .await?;

    Ok(())
}

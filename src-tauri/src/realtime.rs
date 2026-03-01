// realtime.rs — Supabase Realtime WebSocket subscriber
//
// Design Patterns implemented here:
//   - Observer Pattern: this module subscribes to DB change events and reacts
//     to them (Directors observe vote changes; users observe their notifications).
//   - Event Pattern: DB change events are relayed to the Svelte frontend via
//     Tauri's app_handle.emit(), which Svelte consumes with await listen().
//
// Architecture:
//   Supabase Realtime (PostgreSQL logical replication / broadcast channel)
//     → Rust tokio background task (this file)
//         → app_handle.emit("event_name", &payload)
//             → Svelte: await listen("event_name", (event) => { ... })
//
// Source of truth: 00_MASTER_GUIDE.md §3 (Real-Time Flow) and §6 (Real-Time Delivery Rule)

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// ── Event names (must match the listen() calls in Svelte) ─────────────────────

/// Emitted when a new notification is inserted for any user.
/// Svelte listeners re-fetch the notification bell count.
pub const EVENT_NOTIFICATION_NEW: &str = "notification:new";

/// Emitted when a vote session is created or updated.
/// Directors' voting pages react to this to refresh pending votes.
pub const EVENT_VOTE_UPDATED: &str = "vote:updated";

/// Emitted when a new message is delivered on any channel.
/// Svelte listeners refresh the unread message badge.
pub const EVENT_MESSAGE_NEW: &str = "message:new";

// ── Background subscriber entry point ─────────────────────────────────────────

/// Connects to Supabase Realtime via WebSocket and subscribes to table change
/// events. Runs forever in a tokio background task, reconnecting on failure.
///
/// Called once from lib.rs during app setup, after AppState is managed.
pub async fn run_realtime_subscriber(
    app_handle: tauri::AppHandle,
    supabase_url: String,
    anon_key: String,
) {
    // Build WebSocket URL from the Supabase project URL.
    // Supabase Realtime endpoint: wss://<ref>.supabase.co/realtime/v1/websocket
    let base = supabase_url.trim_end_matches('/');
    let ws_url = format!(
        "{}/realtime/v1/websocket?apikey={}&vsn=1.0.0",
        base.replace("https://", "wss://").replace("http://", "ws://"),
        anon_key
    );

    // Reconnect loop — Supabase Realtime connections can drop; always retry.
    loop {
        match connect_async(&ws_url).await {
            Ok((stream, _)) => {
                let (sink, source) = stream.split();

                // mpsc channel lets heartbeat task and initial joins share the
                // write half without needing Arc<Mutex<Sink>>.
                let (write_tx, write_rx) = mpsc::channel::<String>(32);

                // ── Heartbeat (required every 30 s by Supabase Realtime) ───
                let hb_tx = write_tx.clone();
                tokio::spawn(async move {
                    let hb = json!({
                        "event": "heartbeat",
                        "topic": "phoenix",
                        "payload": {},
                        "ref": null
                    })
                    .to_string();
                    loop {
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        if hb_tx.send(hb.clone()).await.is_err() {
                            break;
                        }
                    }
                });

                // ── Subscribe to notifications (Observer: all users) ────────
                let _ = write_tx
                    .send(
                        json!({
                            "event": "phx_join",
                            "topic": "realtime:public:notifications",
                            "payload": {
                                "config": {
                                    "broadcast": { "self": false },
                                    "postgres_changes": [{
                                        "event": "INSERT",
                                        "schema": "public",
                                        "table": "notifications"
                                    }]
                                }
                            },
                            "ref": "sub_notifications"
                        })
                        .to_string(),
                    )
                    .await;

                // ── Subscribe to vote_sessions (Observer: Directors) ────────
                let _ = write_tx
                    .send(
                        json!({
                            "event": "phx_join",
                            "topic": "realtime:public:vote_sessions",
                            "payload": {
                                "config": {
                                    "postgres_changes": [{
                                        "event": "*",
                                        "schema": "public",
                                        "table": "vote_sessions"
                                    }]
                                }
                            },
                            "ref": "sub_votes"
                        })
                        .to_string(),
                    )
                    .await;

                // ── Subscribe to messages (Observer: all recipients) ────────
                let _ = write_tx
                    .send(
                        json!({
                            "event": "phx_join",
                            "topic": "realtime:public:messages",
                            "payload": {
                                "config": {
                                    "postgres_changes": [{
                                        "event": "INSERT",
                                        "schema": "public",
                                        "table": "messages"
                                    }]
                                }
                            },
                            "ref": "sub_messages"
                        })
                        .to_string(),
                    )
                    .await;

                // ── Write task: drains mpsc channel → WebSocket sink ───────
                let mut sink = sink;
                let mut write_rx = write_rx;
                tokio::spawn(async move {
                    while let Some(msg) = write_rx.recv().await {
                        if sink.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                });

                // ── Read loop: relay DB events to Svelte ───────────────────
                let mut source = source;
                while let Some(result) = source.next().await {
                    match result {
                        Ok(Message::Text(text)) => {
                            handle_realtime_message(&app_handle, &text);
                        }
                        Ok(Message::Close(_)) | Err(_) => {
                            break;
                        }
                        _ => {} // ping/pong/binary — ignore
                    }
                }

                eprintln!("[realtime] WebSocket stream ended — reconnecting in 5 s");
            }
            Err(e) => {
                eprintln!("[realtime] Connection failed: {} — retrying in 5 s", e);
            }
        }

        // Back-off before reconnect to avoid hammering the server.
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// ── Message dispatch ──────────────────────────────────────────────────────────

/// Parses a raw Supabase Realtime Phoenix Channels message and emits the
/// corresponding Tauri event so Svelte listeners can react.
fn handle_realtime_message(app_handle: &tauri::AppHandle, raw: &str) {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) else {
        return;
    };

    // Only process `postgres_changes` payloads — ignore phx_reply, heartbeat, etc.
    if val["event"].as_str() != Some("postgres_changes") {
        return;
    }

    let table = val["payload"]["data"]["table"].as_str().unwrap_or("");
    let event_type = val["payload"]["data"]["type"].as_str().unwrap_or("");
    let record = &val["payload"]["data"]["record"];

    let tauri_event = match (table, event_type) {
        ("notifications", "INSERT") => EVENT_NOTIFICATION_NEW,
        ("vote_sessions", _)        => EVENT_VOTE_UPDATED,
        ("messages", "INSERT")      => EVENT_MESSAGE_NEW,
        _                           => return,
    };

    // Emit to ALL Tauri windows. Svelte listeners filter by user_id if needed.
    let _ = app_handle.emit(tauri_event, record);
}

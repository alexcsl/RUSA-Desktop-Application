// lib.rs — Application entry point and module declarations
// Source of truth: AUTH_GUIDE.md §4.6 + 00_MASTER_GUIDE.md §3

pub mod audit;
pub mod auth;
pub mod commands;
pub mod error;
pub mod state;

use commands::auth::{get_current_session, login, logout};
use commands::data_analysts::{
    get_analyst_inbox, get_data_request_detail, get_data_response, get_my_data_requests,
    process_data_request, submit_data_request, submit_data_response,
};
use commands::directors::{
    approve_closure_request, assign_task, create_event, create_meeting,
    create_personnel_account, decide_broadcast_request, decide_data_request,
    decide_experiment_proposal, decide_final_document, decide_test_proposal,
    flag_budget_report, get_broadcast_request_queue, get_data_request_queue,
    get_experiment_proposal_queue, get_final_document_queue,
    get_math_results_for_director, get_test_proposal_queue,
    get_event_documents, get_events, get_financial_queue, get_meetings,
    get_message_recipients, get_notifications, get_outbound_review_queue,
    get_personnel_list,
    get_security_messages, get_subordinate_tasks, get_territories,
    mark_message_read, mark_notification_read, recall_message,
    relocate_personnel, rename_territory,
    review_help_request, review_help_response, review_outbound_data_response,
    send_emergency_broadcast, send_informational_broadcast,
    send_security_message, submit_broadcast_request, terminate_personnel,
    terminate_personnel_account, update_personnel_account, upload_event_document,
};
use commands::engineers::{
    eng_get_approved_tests, eng_get_experiment_archive, eng_get_experiment_detail,
    eng_get_my_help_requests, eng_get_my_tasks, eng_get_my_test_proposals,
    eng_get_progress_reports, eng_get_species_archive, eng_log_daily_experiment,
    eng_propose_new_test, eng_submit_experiment_conclusion, eng_submit_help_request,
    eng_submit_progress_report,
};
use commands::messaging::{
    add_group_member, create_messaging_group, delete_message, delete_messaging_group,
    get_eligible_recipients, get_group_members, get_inbox, get_messaging_groups,
    get_sent_messages, get_message_detail, get_unread_counts, msg_mark_read,
    msg_recall, remove_group_member, send_message,
};
use commands::scientists::{
    get_approved_tests, get_archive, get_experiment_archive, get_experiment_detail,
    get_final_documents, get_math_results, get_my_tasks, get_my_test_proposals,
    log_experiment_session, propose_experiment, propose_new_species, propose_new_test,
    submit_experiment_conclusion, submit_final_document, submit_help_request,
    submit_math_results,
};
use commands::voting::{
    admin_override_vote, admin_terminate_vote, cast_vote, change_vote,
    get_pending_votes, get_vote_session_detail, get_vote_sessions,
    initiate_ad_hoc_vote,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use state::AppState;
use std::str::FromStr;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Load environment variables from src-tauri/.env
            dotenvy::dotenv().ok();

            tauri::async_runtime::block_on(async {
                // ── PostgreSQL (Supabase) ──────────────────────────────────
                let db_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in src-tauri/.env");

                // statement_cache_capacity(0) is required when using Supabase's
                // pgBouncer pooler (port 6543, transaction mode).
                let connect_opts = PgConnectOptions::from_str(&db_url)
                    .expect("Invalid DATABASE_URL format")
                    .statement_cache_capacity(0);

                let db_pool = PgPoolOptions::new()
                    .max_connections(5)
                    .connect_with(connect_opts)
                    .await
                    .expect("Failed to connect to Supabase PostgreSQL");

                // ── Redis ──────────────────────────────────────────────────
                let redis_url = std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

                let redis_client = redis::Client::open(redis_url)
                    .expect("Failed to create Redis client");

                // ── Supabase Storage ───────────────────────────────────────
                let supabase_storage_url = std::env::var("SUPABASE_STORAGE_URL")
                    .unwrap_or_default();

                let supabase_service_key = std::env::var("SUPABASE_SERVICE_KEY")
                    .unwrap_or_default();

                // Generate a valid service_role JWT from the JWT secret.
                // Supabase Storage requires a proper JWT, not the raw secret.
                let supabase_service_jwt = if supabase_service_key.is_empty() {
                    String::new()
                } else {
                    use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
                    let header = Header::new(Algorithm::HS256);
                    let now = chrono::Utc::now().timestamp();
                    let claims = serde_json::json!({
                        "role": "service_role",
                        "iss": "supabase",
                        "iat": now,
                        "exp": now + 315_360_000, // ~10 years
                    });
                    encode(&header, &claims, &EncodingKey::from_secret(supabase_service_key.as_bytes()))
                        .expect("Failed to generate Supabase service_role JWT from SUPABASE_SERVICE_KEY")
                };

                // ── Manage AppState singleton ──────────────────────────────
                app.manage(AppState {
                    current_user: tokio::sync::Mutex::new(None),
                    db_pool,
                    redis_client,
                    supabase_storage_url,
                    supabase_service_jwt,
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            login,
            logout,
            get_current_session,
            // Account management
            create_personnel_account,
            terminate_personnel_account,
            update_personnel_account,
            // Voting
            cast_vote,
            change_vote,
            get_pending_votes,
            get_vote_session_detail,
            get_vote_sessions,
            initiate_ad_hoc_vote,
            admin_override_vote,
            admin_terminate_vote,
            // Meetings
            create_meeting,
            get_meetings,
            // Financial
            get_financial_queue,
            flag_budget_report,
            // Personnel relocation
            relocate_personnel,
            // Task management
            assign_task,
            get_subordinate_tasks,
            approve_closure_request,
            review_help_request,
            review_help_response,
            // Experiment proposals (Artificer / Observer)
            get_experiment_proposal_queue,
            decide_experiment_proposal,
            get_math_results_for_director,
            // Test proposal review (Artificer / Observer)
            get_test_proposal_queue,
            decide_test_proposal,
            // Final document review (Artificer / Observer)
            get_final_document_queue,
            decide_final_document,
            // Territory
            rename_territory,
            get_territories,
            // Broadcasts
            submit_broadcast_request,
            get_broadcast_request_queue,
            decide_broadcast_request,
            send_emergency_broadcast,
            send_informational_broadcast,
            // Data requests (Statistician)
            get_data_request_queue,
            decide_data_request,
            get_outbound_review_queue,
            review_outbound_data_response,
            // Data Analysts (UC-DRQ-01, UC-DA-01/02/03)
            submit_data_request,
            get_my_data_requests,
            get_data_request_detail,
            get_analyst_inbox,
            process_data_request,
            submit_data_response,
            get_data_response,
            // Events (Coordinator)
            create_event,
            get_events,
            upload_event_document,
            get_event_documents,
            // Termination
            terminate_personnel,
            // Notifications
            get_notifications,
            mark_notification_read,
            // Personnel listing
            get_personnel_list,
            // Secure messaging (directors — legacy security channel)
            send_security_message,
            get_security_messages,
            get_message_recipients,
            recall_message,
            mark_message_read,
            // Universal messaging system (all channels)
            send_message,
            get_inbox,
            get_sent_messages,
            get_message_detail,
            msg_mark_read,
            msg_recall,
            delete_message,
            get_unread_counts,
            get_eligible_recipients,
            // Messaging groups
            create_messaging_group,
            get_messaging_groups,
            get_group_members,
            add_group_member,
            remove_group_member,
            delete_messaging_group,
            // Scientists subsystem (UC-GS-01..03, UC-MA-01, UC-PH-01..08, UC-CH-01..07, UC-BIO-01..07)
            get_my_tasks,
            submit_help_request,
            submit_math_results,
            propose_experiment,
            log_experiment_session,
            submit_experiment_conclusion,
            submit_final_document,
            get_archive,
            get_experiment_archive,
            get_experiment_detail,
            get_math_results,
            get_approved_tests,
            propose_new_test,
            propose_new_species,
            get_final_documents,
            get_my_test_proposals,
            // Engineers subsystem (UC-GE-01..03, UC-AGE-01..06, UC-BE-01..05)
            eng_get_my_tasks,
            eng_submit_progress_report,
            eng_get_progress_reports,
            eng_submit_help_request,
            eng_get_approved_tests,
            eng_propose_new_test,
            eng_get_my_test_proposals,
            eng_log_daily_experiment,
            eng_submit_experiment_conclusion,
            eng_get_species_archive,
            eng_get_experiment_archive,
            eng_get_experiment_detail,
            eng_get_my_help_requests,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

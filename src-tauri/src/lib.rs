// lib.rs — Application entry point and module declarations
// Source of truth: AUTH_GUIDE.md §4.6 + 00_MASTER_GUIDE.md §3

pub mod audit;
pub mod auth;
pub mod commands;
pub mod error;
pub mod realtime;
pub mod reminders;
pub mod state;

use commands::administrator::{
    admin_get_system_stats, admin_get_audit_log, admin_get_all_users,
    admin_toggle_user_status, admin_reset_password, admin_get_recent_activity,
    admin_get_roles, admin_get_base_locations,
};
use commands::astronauts::{
    ast_assign_mission, ast_get_all_missions, ast_get_all_status_reports,
    ast_get_colleague_counters, ast_get_completion_requests_taskmaster,
    ast_get_completion_requests_wanderer, ast_get_evidence_urls,
    ast_get_mission_detail, ast_get_my_missions, ast_get_personal_journal,
    ast_process_completion_request, ast_submit_completion_request,
    ast_submit_status_report, ast_taskmaster_decide_completion,
};
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
    dir_get_daily_security_reports,
};
use commands::engineers::{
    eng_get_approved_tests, eng_get_experiment_archive, eng_get_experiment_detail,
    eng_get_my_help_requests, eng_get_my_tasks, eng_get_my_test_proposals,
    eng_get_progress_reports, eng_get_species_archive, eng_log_daily_experiment,
    eng_propose_new_test, eng_submit_experiment_conclusion, eng_submit_help_request,
    eng_submit_progress_report,
};
use commands::medical::{
    med_add_inventory, med_allocate_shift, med_delete_shift,
    med_get_all_shifts, med_get_budget_requests, med_get_expenditure_reports,
    med_get_inventory, med_get_my_shifts, med_get_patient_record,
    med_get_patients, med_get_staff_list, med_get_user_lookup,
    med_log_treatment, med_register_patient, med_remove_inventory,
    med_submit_budget_request, med_submit_expenditure_report,
    med_update_inventory,
};
use commands::sanitary::{
    san_add_inventory, san_allocate_shift, san_assign_recruit, san_assign_task,
    san_create_disposal_doc, san_create_wastewater_doc, san_delete_shift,
    san_get_all_shifts, san_get_all_tasks, san_get_budget_requests,
    san_get_disposal_docs, san_get_divisions, san_get_expenditure_reports,
    san_get_inspection_reports, san_get_inventory, san_get_inventory_logs,
    san_get_my_schedule, san_get_my_tasks, san_get_my_transfers,
    san_get_staff_roster, san_get_transfer_requests, san_get_wastewater_docs,
    san_log_inventory_action, san_remove_inventory, san_review_transfer_request,
    san_set_division_quota, san_submit_budget_request, san_submit_expenditure_report,
    san_submit_inspection_report, san_submit_transfer_request, san_update_disposal_doc,
    san_update_inventory, san_update_task_status, san_update_wastewater_doc,
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
use commands::security::{
    sec_assign_staff_to_incident, sec_create_incident_report, sec_get_incident_archive,
    sec_get_my_broadcast_requests, sec_get_my_daily_reports, sec_get_security_messages,
    sec_get_security_personnel, sec_send_security_message, sec_submit_broadcast_request,
    sec_submit_daily_report,
};
use commands::space_station::{
    sst_add_annotation, sst_add_to_archive, sst_delete_annotation, sst_get_annotations,
    sst_get_archive, sst_get_inventory, sst_get_maps, sst_get_personnel,
    sst_get_public_stations, sst_get_published_map, sst_get_stations,
    sst_get_supply_requests, sst_log_arrival, sst_log_departure, sst_manage_inventory,
    sst_propose_abandonment, sst_publish_map, sst_report_finding_to_security,
    sst_submit_supply_request, sst_upload_map,
};
use commands::settlers::{
    stl_assign_task, stl_forward_to_directors, stl_get_dashboard, stl_get_incoming_queue,
    stl_get_my_tasks, stl_get_residence, stl_get_settlement_inventory, stl_get_settlement_members,
    stl_get_task_detail,
    stl_log_building_health, stl_log_farm_health, stl_manage_inventory, stl_reject_incoming,
    stl_request_abandonment, stl_request_farming_supplies, stl_request_materials,
    stl_request_repatriation, stl_set_house_arrest, stl_submit_anomaly_report,
    stl_submit_commander_anomaly, stl_submit_commander_supply, stl_submit_complaint,
    stl_submit_construction_report, stl_submit_progress_report, stl_submit_supply_request,
};
use commands::psychiatry::{
    psy_create_patient_record, psy_get_user_directory, psy_get_my_patients,
    psy_get_patient_detail, psy_log_appointment, psy_get_appointments,
    psy_update_recovery_log, psy_get_recovery_logs, psy_manage_schedule,
    psy_get_schedule, psy_delete_schedule_slot, psy_get_patient_index,
    psy_schedule_appointment, psy_request_schedule_access,
    psy_assistant_get_patients, psy_assistant_get_recovery_log,
    psy_assistant_get_schedule, psy_assistant_get_appointments,
    psy_grant_schedule_access, psy_get_access_settings,
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

                // ── Supabase Realtime ──────────────────────────────────────
                // Used for the Observer + Event patterns:
                //   notifications → EVENT_NOTIFICATION_NEW
                //   vote_sessions → EVENT_VOTE_UPDATED
                //   messages      → EVENT_MESSAGE_NEW
                //
                // SUPABASE_URL  — base project URL, e.g. https://xyz.supabase.co
                // SUPABASE_ANON_KEY — public anon key (safe to use in desktop apps;
                //   RLS policies on Supabase enforce row-level security).
                let supabase_url = std::env::var("SUPABASE_URL").unwrap_or_default();
                let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY").unwrap_or_default();

                // ── Manage AppState singleton ──────────────────────────────
                app.manage(AppState {
                    current_user: tokio::sync::Mutex::new(None),
                    db_pool,
                    redis_client,
                    supabase_storage_url,
                    supabase_service_jwt,
                });

                // ── Spawn Supabase Realtime subscriber ─────────────────────
                // Runs as a persistent background tokio task. Connects to the
                // Supabase Realtime WebSocket and emits Tauri events to Svelte
                // whenever watched tables change. Reconnects automatically.
                if !supabase_url.is_empty() && !supabase_anon_key.is_empty() {
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn(
                        crate::realtime::run_realtime_subscriber(
                            handle,
                            supabase_url,
                            supabase_anon_key,
                        )
                    );
                }

                // ── Spawn sub-quorum voting reminder task ───────────────────
                // Fires every 15 minutes to notify Directors who haven't voted
                // on open sessions that are still below quorum.
                // Rule: 00_MASTER_GUIDE.md §6 "15-minute push reminders"
                {
                    let handle = app.handle().clone();
                    let pool = app.state::<AppState>().db_pool.clone();
                    tauri::async_runtime::spawn(
                        crate::reminders::run_voting_reminders(handle, pool)
                    );
                }
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
            // Astronauts subsystem (UC-AS-01..06, UC-WAN-01..03, UC-TM-03)
            ast_get_my_missions,
            ast_get_mission_detail,
            ast_submit_status_report,
            ast_submit_completion_request,
            ast_get_personal_journal,
            ast_get_colleague_counters,
            ast_get_evidence_urls,
            ast_assign_mission,
            ast_get_all_missions,
            ast_process_completion_request,
            ast_get_all_status_reports,
            ast_get_completion_requests_wanderer,
            ast_taskmaster_decide_completion,
            ast_get_completion_requests_taskmaster,
            // Settlers subsystem (UC-PS-01..06, UC-SC-01..08, UC-CE-01..04, UC-FA-01..02, UC-TS-01..03)
            stl_get_my_tasks,
            stl_get_task_detail,
            stl_submit_progress_report,
            stl_submit_anomaly_report,
            stl_submit_complaint,
            stl_submit_supply_request,
            stl_get_settlement_inventory,
            stl_get_settlement_members,
            stl_assign_task,
            stl_get_incoming_queue,
            stl_reject_incoming,
            stl_forward_to_directors,
            stl_submit_commander_anomaly,
            stl_request_abandonment,
            stl_request_repatriation,
            stl_set_house_arrest,
            stl_get_dashboard,
            stl_submit_commander_supply,
            stl_manage_inventory,
            stl_submit_construction_report,
            stl_request_materials,
            stl_log_building_health,
            stl_get_residence,
            stl_request_farming_supplies,
            stl_log_farm_health,
            // Security Teams subsystem (UC-SH-01..05, UC-SS-01..02)
            sec_create_incident_report,
            sec_get_incident_archive,
            sec_assign_staff_to_incident,
            sec_submit_daily_report,
            sec_submit_broadcast_request,
            sec_get_my_broadcast_requests,
            sec_send_security_message,
            sec_get_security_messages,
            sec_get_security_personnel,
            sec_get_my_daily_reports,
            // Guardian daily security reports view
            dir_get_daily_security_reports,
            // Space Station Settlers subsystem (UC-SSS-01..08, UC-SSV-01)
            sst_report_finding_to_security,
            sst_add_to_archive,
            sst_get_archive,
            sst_manage_inventory,
            sst_get_inventory,
            sst_submit_supply_request,
            sst_get_supply_requests,
            sst_upload_map,
            sst_get_maps,
            sst_publish_map,
            sst_add_annotation,
            sst_delete_annotation,
            sst_get_annotations,
            sst_get_published_map,
            sst_log_arrival,
            sst_log_departure,
            sst_get_personnel,
            sst_propose_abandonment,
            sst_get_stations,
            sst_get_public_stations,
            // Medical Department subsystem (UC-MED-01..04, UC-HOM-01..03)
            med_register_patient,
            med_log_treatment,
            med_get_patients,
            med_get_patient_record,
            med_get_inventory,
            med_add_inventory,
            med_update_inventory,
            med_remove_inventory,
            med_get_my_shifts,
            med_get_staff_list,
            med_get_all_shifts,
            med_allocate_shift,
            med_delete_shift,
            med_submit_budget_request,
            med_get_budget_requests,
            med_submit_expenditure_report,
            med_get_expenditure_reports,
            med_get_user_lookup,
            // Sanitary Department subsystem (UC-HS-01..08, UC-STAS-01..04, UC-IC-01, UC-DC-01, UC-WC-01)
            san_get_divisions,
            san_set_division_quota,
            san_get_staff_roster,
            san_get_all_shifts,
            san_allocate_shift,
            san_delete_shift,
            san_assign_task,
            san_get_all_tasks,
            san_get_transfer_requests,
            san_review_transfer_request,
            san_assign_recruit,
            san_get_inventory,
            san_add_inventory,
            san_update_inventory,
            san_remove_inventory,
            san_get_my_tasks,
            san_update_task_status,
            san_get_my_schedule,
            san_submit_transfer_request,
            san_get_my_transfers,
            san_log_inventory_action,
            san_get_inventory_logs,
            san_submit_inspection_report,
            san_get_inspection_reports,
            san_create_disposal_doc,
            san_update_disposal_doc,
            san_get_disposal_docs,
            san_create_wastewater_doc,
            san_update_wastewater_doc,
            san_get_wastewater_docs,
            san_submit_budget_request,
            san_get_budget_requests,
            san_submit_expenditure_report,
            san_get_expenditure_reports,
            // Psychiatry Division subsystem (UC-PSY-01..05, UC-PA-01..04, UC-PAT-01)
            psy_create_patient_record,
            psy_get_user_directory,
            psy_get_my_patients,
            psy_get_patient_detail,
            psy_log_appointment,
            psy_get_appointments,
            psy_update_recovery_log,
            psy_get_recovery_logs,
            psy_manage_schedule,
            psy_get_schedule,
            psy_delete_schedule_slot,
            psy_get_patient_index,
            psy_schedule_appointment,
            psy_request_schedule_access,
            psy_assistant_get_patients,
            psy_assistant_get_recovery_log,
            psy_assistant_get_schedule,
            psy_assistant_get_appointments,
            psy_grant_schedule_access,
            psy_get_access_settings,
            // Administrator subsystem (UC-ADM-01..06 + system management)
            admin_get_system_stats,
            admin_get_audit_log,
            admin_get_all_users,
            admin_toggle_user_status,
            admin_reset_password,
            admin_get_recent_activity,
            admin_get_roles,
            admin_get_base_locations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Per 00_MASTER_GUIDE.md § Architecture:
// "The Svelte frontend NEVER connects to Supabase directly.
//  All data flows through Rust Tauri commands."
//
// Authentication, DB queries, and Storage are all handled exclusively
// in Rust (src-tauri/src/commands/). Use `invoke()` from @tauri-apps/api/core.
//
// This file is intentionally left empty.
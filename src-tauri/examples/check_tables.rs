// HELPER SCRIPT DOANG, DELETE LATER IF NOT NEEDED
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::Row;
use std::str::FromStr;

fn main() {
    dotenvy::from_filename("d:\\Codin\\TPA\\Desktop\\Code\\rusa\\src-tauri\\.env").ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {

    let connect_opts = PgConnectOptions::from_str(&db_url)
        .expect("Invalid DATABASE_URL")
        .statement_cache_capacity(0);

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(connect_opts)
        .await
        .expect("Failed to connect");

    println!("Connected to Supabase!");

    let rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name"
    )
    .fetch_all(&pool)
    .await
    .expect("Query failed");

    if rows.is_empty() {
        println!("No tables found in public schema.");
    } else {
        println!("\nExisting tables in public schema:");
        for row in &rows {
            let name: String = row.get("table_name");
            println!("  - {}", name);
        }
    }

    pool.close().await;
    });
}

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::fs;
use tauri::AppHandle;
use tauri::Manager;

pub async fn init_db(app_handle: &AppHandle) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    let db_path = app_dir.join("connectivity_monitor.db");

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Initial purge
    if let Err(e) = purge_old_data(&pool).await {
        eprintln!("failed to purge old data: {}", e);
    }

    Ok(pool)
}

pub async fn purge_old_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let result_checks =
        sqlx::query("DELETE FROM checks WHERE timestamp < datetime('now', '-30 days')")
            .execute(pool)
            .await?;

    let result_transitions =
        sqlx::query("DELETE FROM transitions WHERE timestamp < datetime('now', '-30 days')")
            .execute(pool)
            .await?;

    let result_errors =
        sqlx::query("DELETE FROM app_errors WHERE timestamp < datetime('now', '-30 days')")
            .execute(pool)
            .await?;

    println!(
        "Purge completed: {} checks, {} transitions, {} errors removed",
        result_checks.rows_affected(),
        result_transitions.rows_affected(),
        result_errors.rows_affected()
    );

    Ok(())
}

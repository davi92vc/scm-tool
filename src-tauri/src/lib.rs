// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use tauri::{Manager, State};
use sqlx::SqlitePool;
use crate::models::Device;
use crate::repository::Repository;
use crate::domain::DeviceService;
use crate::monitor::MonitoringEngine;

mod db;
mod models;
mod repository;
mod domain;
mod monitor;

#[tauri::command]
async fn get_devices(pool: State<'_, SqlitePool>) -> Result<Vec<Device>, String> {
    let repo = Repository::new(pool.inner().clone());
    let service = DeviceService::new(repo);
    service.get_all_devices().await
}

#[tauri::command]
async fn add_device(
    name: String,
    ip: String,
    pool: State<'_, SqlitePool>,
    engine: State<'_, MonitoringEngine>
) -> Result<i64, String> {
    let repo = Repository::new(pool.inner().clone());
    let service = DeviceService::new(repo);
    let id = service.create_device(&name, &ip).await?;
    
    // Sync monitor
    engine.sync_devices().await?;
    
    Ok(id)
}

#[tauri::command]
async fn remove_device(
    id: i64,
    pool: State<'_, SqlitePool>,
    engine: State<'_, MonitoringEngine>
) -> Result<(), String> {
    let repo = Repository::new(pool.inner().clone());
    let service = DeviceService::new(repo);
    service.delete_device(id).await?;
    
    // Sync monitor
    engine.sync_devices().await?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_devices,
            add_device,
            remove_device
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pool = db::init_db(&app_handle).await.expect("failed to initialize database");
                let repo = Repository::new(pool.clone());
                let engine = MonitoringEngine::new(app_handle.clone(), repo);
                
                // Initial sync
                engine.sync_devices().await.expect("failed initial monitor sync");
                
                app_handle.manage(pool);
                app_handle.manage(engine);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


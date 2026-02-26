// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use crate::domain::DeviceService;
use crate::models::Device;
use crate::monitor::MonitoringEngine;
use crate::repository::Repository;
use sqlx::SqlitePool;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, State};

mod db;
mod domain;
mod models;
mod monitor;
mod repository;

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
    engine: State<'_, MonitoringEngine>,
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
    engine: State<'_, MonitoringEngine>,
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
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_devices,
            add_device,
            remove_device
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Tray setup
            let open = MenuItem::with_id(&app_handle, "open", "Abrir", true, None::<&str>)?;
            let quit = MenuItem::with_id(&app_handle, "quit", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(&app_handle, &[&open, &quit])?;

            let _tray = TrayIconBuilder::with_id("tray")
                .menu(&menu)
                .on_menu_event(|app: &tauri::AppHandle, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            tauri::async_runtime::block_on(async move {
                let pool = db::init_db(&app_handle)
                    .await
                    .expect("failed to initialize database");
                let repo = Repository::new(pool.clone());
                let engine = MonitoringEngine::new(app_handle.clone(), repo);

                // Initial sync
                engine
                    .sync_devices()
                    .await
                    .expect("failed initial monitor sync");

                app_handle.manage(pool);
                app_handle.manage(engine);
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Intercept close and hide window instead of exiting
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


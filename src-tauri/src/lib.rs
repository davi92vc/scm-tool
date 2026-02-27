// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use crate::domain::DeviceService;
use crate::models::{AppError, AppSettings, Device};
use crate::monitor::MonitoringEngine;
use crate::repository::Repository;
use sqlx::SqlitePool;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, State};
use tauri_plugin_autostart::ManagerExt;

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
async fn get_app_errors(pool: State<'_, SqlitePool>) -> Result<Vec<AppError>, String> {
    let repo = Repository::new(pool.inner().clone());
    let service = DeviceService::new(repo);
    service.get_app_errors(25).await
}

#[tauri::command]
async fn get_settings(
    pool: State<'_, SqlitePool>,
    app_handle: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let repo = Repository::new(pool.inner().clone());
    let service = DeviceService::new(repo);
    let mut settings = service.get_or_create_app_settings().await?;

    if let Ok(autostart_enabled) = app_handle.autolaunch().is_enabled() {
        if settings.autostart_enabled != autostart_enabled {
            settings = service
                .update_app_settings(
                    settings.online_interval_sec,
                    settings.offline_interval_sec,
                    autostart_enabled,
                )
                .await?;
        }
    }

    Ok(settings)
}

#[tauri::command]
async fn update_settings(
    online_interval_sec: i64,
    offline_interval_sec: i64,
    autostart_enabled: bool,
    pool: State<'_, SqlitePool>,
    engine: State<'_, MonitoringEngine>,
    app_handle: tauri::AppHandle,
) -> Result<AppSettings, String> {
    DeviceService::validate_monitoring_intervals(online_interval_sec, offline_interval_sec)?;

    let current_autostart = app_handle
        .autolaunch()
        .is_enabled()
        .map_err(|e| e.to_string())?;

    if autostart_enabled && !current_autostart {
        app_handle
            .autolaunch()
            .enable()
            .map_err(|e| e.to_string())?;
    } else if !autostart_enabled && current_autostart {
        if let Err(error) = app_handle.autolaunch().disable() {
            let message = error.to_string();
            if !message.contains("os error 2") {
                return Err(message);
            }
        }
    }

    let effective_autostart = app_handle
        .autolaunch()
        .is_enabled()
        .map_err(|e| e.to_string())?;

    let repo = Repository::new(pool.inner().clone());
    let service = DeviceService::new(repo);
    let settings = service
        .update_app_settings(
            online_interval_sec,
            offline_interval_sec,
            effective_autostart,
        )
        .await?;

    engine
        .update_intervals(
            settings.online_interval_sec as u64,
            settings.offline_interval_sec as u64,
        )
        .await?;

    Ok(settings)
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

#[tauri::command]
async fn update_device(
    id: i64,
    name: String,
    ip: String,
    pool: State<'_, SqlitePool>,
    engine: State<'_, MonitoringEngine>,
) -> Result<(), String> {
    let repo = Repository::new(pool.inner().clone());
    let service = DeviceService::new(repo);
    service.update_device(id, &name, &ip).await?;

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
            get_app_errors,
            get_settings,
            update_settings,
            add_device,
            update_device,
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

            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.hide();
            }

            tauri::async_runtime::block_on(async move {
                let pool = db::init_db(&app_handle)
                    .await
                    .expect("failed to initialize database");

                let settings_service = DeviceService::new(Repository::new(pool.clone()));
                let mut settings = settings_service
                    .get_or_create_app_settings()
                    .await
                    .expect("failed to load app settings");

                if let Ok(autostart_enabled) = app_handle.autolaunch().is_enabled() {
                    if settings.autostart_enabled != autostart_enabled {
                        settings = settings_service
                            .update_app_settings(
                                settings.online_interval_sec,
                                settings.offline_interval_sec,
                                autostart_enabled,
                            )
                            .await
                            .expect("failed to sync autostart setting");
                    }
                }

                let repo = Repository::new(pool.clone());
                let engine = MonitoringEngine::new(app_handle.clone(), repo);

                engine
                    .set_intervals(
                        settings.online_interval_sec as u64,
                        settings.offline_interval_sec as u64,
                    )
                    .await;

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


use crate::models::{Check, Device};
use crate::repository::Repository;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tauri::image::Image;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

const TRAY_ICON_DEFAULT: &[u8] = include_bytes!("../icons/32x32.png");
const TRAY_ICON_GREEN: &[u8] = include_bytes!("../icons/tray-green.png");
const TRAY_ICON_RED: &[u8] = include_bytes!("../icons/tray-red.png");

#[derive(Serialize)]
struct NotificationErrorEvent {
    source: String,
    message: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TrayStatus {
    Neutral,
    AllOnline,
    HasOffline,
}

pub struct MonitoringEngine {
    app_handle: AppHandle,
    repository: Arc<Repository>,
    active_monitors: Arc<Mutex<HashMap<i64, MonitorEntry>>>,
    device_states: Arc<Mutex<HashMap<i64, Option<bool>>>>,
    tray_status: Arc<Mutex<TrayStatus>>,
    ping_client: Arc<Client>,
}

struct MonitorEntry {
    handle: tokio::task::JoinHandle<()>,
    ip: String,
    name: String,
}

impl MonitoringEngine {
    pub fn new(app_handle: AppHandle, repository: Repository) -> Self {
        let client = Client::new(&Config::default())
            .expect("failed to create ICMP client (might need admin privileges on some systems)");

        Self {
            app_handle,
            repository: Arc::new(repository),
            active_monitors: Arc::new(Mutex::new(HashMap::new())),
            device_states: Arc::new(Mutex::new(HashMap::new())),
            tray_status: Arc::new(Mutex::new(TrayStatus::Neutral)),
            ping_client: Arc::new(client),
        }
    }

    pub async fn sync_devices(&self) -> Result<(), String> {
        let devices = self
            .repository
            .get_all_devices()
            .await
            .map_err(|e| e.to_string())?;

        let mut active_monitors = self.active_monitors.lock().await;
        let mut device_states = self.device_states.lock().await;

        // Stop monitors for removed devices
        let current_ids: Vec<i64> = devices.iter().filter_map(|d| d.id).collect();
        device_states.retain(|id, _| current_ids.contains(id));
        active_monitors.retain(|id, entry| {
            if !current_ids.contains(id) {
                entry.handle.abort();
                false
            } else {
                true
            }
        });

        // Start monitors for new devices and restart monitors when edited
        for device in devices {
            let id = device.id.expect("Device ID is required");

            let should_restart = active_monitors
                .get(&id)
                .map(|entry| entry.ip != device.ip || entry.name != device.name)
                .unwrap_or(false);

            if should_restart {
                if let Some(old_entry) = active_monitors.remove(&id) {
                    old_entry.handle.abort();
                }
                device_states.insert(id, None);
            }

            if !active_monitors.contains_key(&id) {
                let repo = Arc::clone(&self.repository);
                let app_handle = self.app_handle.clone();
                let client = Arc::clone(&self.ping_client);
                let states = Arc::clone(&self.device_states);
                let tray_status = Arc::clone(&self.tray_status);
                let monitor_name = device.name.clone();
                let monitor_ip = device.ip.clone();
                device_states.entry(id).or_insert(None);
                let handle = tokio::spawn(async move {
                    run_monitor(app_handle, repo, client, states, tray_status, device).await;
                });
                active_monitors.insert(
                    id,
                    MonitorEntry {
                        handle,
                        ip: monitor_ip,
                        name: monitor_name,
                    },
                );
            }
        }

        drop(device_states);
        drop(active_monitors);

        refresh_tray_icon(
            &self.app_handle,
            Arc::clone(&self.device_states),
            Arc::clone(&self.tray_status),
        )
        .await;

        Ok(())
    }
}

async fn run_monitor(
    app_handle: AppHandle,
    repo: Arc<Repository>,
    client: Arc<Client>,
    device_states: Arc<Mutex<HashMap<i64, Option<bool>>>>,
    tray_status: Arc<Mutex<TrayStatus>>,
    device: Device,
) {
    let device_id = device.id.unwrap();
    let mut last_status = true;
    let mut state_initialized = false;
    let ip = IpAddr::from_str(&device.ip).expect("invalid IP stored in DB");

    loop {
        let start = std::time::Instant::now();
        let ping_result = perform_ping(&client, ip).await;
        let latency = start.elapsed();

        let (is_online, error) = match ping_result {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e)),
        };

        // Persist check
        let check = Check {
            id: None,
            device_id,
            timestamp: None,
            is_online,
            latency_ms: Some(latency.as_secs_f64() * 1000.0),
            error_msg: error,
        };

        if let Err(e) = repo.insert_check(&check).await {
            eprintln!(
                "Failed to insert check for device {}: {}",
                device.id.unwrap(),
                e
            );
        }

        // Emit check event
        let _ = app_handle.emit("check-event", &check);

        update_device_status_and_tray_icon(
            &app_handle,
            Arc::clone(&device_states),
            Arc::clone(&tray_status),
            device_id,
            Some(is_online),
        )
        .await;

        // Transition logic
        if !state_initialized {
            let status_text = status_label_pt_br(is_online);
            send_device_notification(
                &app_handle,
                &repo,
                &device,
                format!("Status inicial detectado: {}", device.name),
                format!(
                    "O dispositivo {} (IP: {}) está {}.",
                    device.name, device.ip, status_text
                ),
            )
            .await;
            last_status = is_online;
            state_initialized = true;
        } else if is_online != last_status {
            // Detected transition!
            let transition = crate::models::Transition {
                id: None,
                device_id,
                from_status: if last_status {
                    "Online".to_string()
                } else {
                    "Offline".to_string()
                },
                to_status: if is_online {
                    "Online".to_string()
                } else {
                    "Offline".to_string()
                },
                timestamp: None,
            };

            if let Err(e) = repo.insert_transition(&transition).await {
                eprintln!(
                    "Failed to insert transition for device {}: {}",
                    device.id.unwrap(),
                    e
                );
            }

            // Emit transition event
            let _ = app_handle.emit("transition-event", &transition);

            // Send native notification
            let from_status_text = status_label_pt_br(last_status);
            let to_status_text = status_label_pt_br(is_online);
            send_device_notification(
                &app_handle,
                &repo,
                &device,
                format!("Mudança de status: {}", device.name),
                format!(
                    "O dispositivo {} (IP: {}) mudou de {} para {}.",
                    device.name, device.ip, from_status_text, to_status_text
                ),
            )
            .await;

            last_status = is_online;
        }

        // Wait based on status
        let interval = if is_online {
            Duration::from_secs(10)
        } else {
            Duration::from_secs(2)
        };

        sleep(interval).await;
    }
}

async fn send_device_notification(
    app_handle: &AppHandle,
    repo: &Repository,
    device: &Device,
    title: String,
    body: String,
) {
    if let Err(error) = app_handle
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show()
    {
        let error_message = format!(
            "Não foi possível exibir a notificação do dispositivo {} ({}): {}",
            device.name, device.ip, error
        );

        eprintln!("{}", error_message);

        if let Err(repo_error) = repo
            .insert_error(&crate::models::AppError {
                id: None,
                source: "NOTIFICATION".to_string(),
                message: error_message.clone(),
                timestamp: None,
            })
            .await
        {
            eprintln!("Não foi possível registrar o erro de notificação: {}", repo_error);
        }

        let event = NotificationErrorEvent {
            source: "NOTIFICATION".to_string(),
            message: error_message,
        };

        let _ = app_handle.emit("notification-error-event", &event);
    }
}

fn status_label_pt_br(is_online: bool) -> &'static str {
    if is_online {
        "online"
    } else {
        "offline"
    }
}

async fn update_device_status_and_tray_icon(
    app_handle: &AppHandle,
    device_states: Arc<Mutex<HashMap<i64, Option<bool>>>>,
    tray_status: Arc<Mutex<TrayStatus>>,
    device_id: i64,
    is_online: Option<bool>,
) {
    {
        let mut states = device_states.lock().await;
        states.insert(device_id, is_online);
    }

    refresh_tray_icon(app_handle, device_states, tray_status).await;
}

async fn refresh_tray_icon(
    app_handle: &AppHandle,
    device_states: Arc<Mutex<HashMap<i64, Option<bool>>>>,
    tray_status: Arc<Mutex<TrayStatus>>,
) {
    let next_status = {
        let states = device_states.lock().await;
        calculate_tray_status(&states)
    };

    {
        let mut current_status = tray_status.lock().await;
        if *current_status == next_status {
            return;
        }
        *current_status = next_status;
    }

    if let Err(error) = apply_tray_icon(app_handle, next_status) {
        eprintln!("Failed to update tray icon: {}", error);
    }
}

fn calculate_tray_status(states: &HashMap<i64, Option<bool>>) -> TrayStatus {
    if states.is_empty() {
        return TrayStatus::Neutral;
    }

    if states.values().any(|status| status.is_none()) {
        return TrayStatus::Neutral;
    }

    if states
        .values()
        .any(|status| matches!(status, Some(false)))
    {
        return TrayStatus::HasOffline;
    }

    TrayStatus::AllOnline
}

fn apply_tray_icon(app_handle: &AppHandle, status: TrayStatus) -> Result<(), String> {
    let tray = app_handle
        .tray_by_id("tray")
        .ok_or_else(|| "tray not found".to_string())?;

    let icon_bytes = match status {
        TrayStatus::Neutral => TRAY_ICON_DEFAULT,
        TrayStatus::AllOnline => TRAY_ICON_GREEN,
        TrayStatus::HasOffline => TRAY_ICON_RED,
    };

    let icon = Image::from_bytes(icon_bytes)
        .map(|image| image.to_owned())
        .map_err(|e| e.to_string())?;

    tray.set_icon(Some(icon)).map_err(|e| e.to_string())
}

async fn perform_ping(client: &Client, ip: IpAddr) -> Result<(), String> {
    let mut pinger = client.pinger(ip, PingIdentifier(0)).await;
    pinger
        .ping(PingSequence(0), &[])
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

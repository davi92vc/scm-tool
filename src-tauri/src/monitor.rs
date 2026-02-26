use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tauri::{AppHandle, Emitter};
use crate::models::{Check, Device};
use crate::repository::Repository;

pub struct MonitoringEngine {
    app_handle: AppHandle,
    repository: Arc<Repository>,
    active_monitors: Arc<Mutex<HashMap<i64, tokio::task::JoinHandle<()>>>>,
}

impl MonitoringEngine {
    pub fn new(app_handle: AppHandle, repository: Repository) -> Self {
        Self {
            app_handle,
            repository: Arc::new(repository),
            active_monitors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn sync_devices(&self) -> Result<(), String> {
        let devices = self.repository.get_all_devices().await
            .map_err(|e| e.to_string())?;

        let mut active_monitors = self.active_monitors.lock().await;

        // Stop monitors for removed devices
        let current_ids: Vec<i64> = devices.iter().filter_map(|d| d.id).collect();
        active_monitors.retain(|id, handle| {
            if !current_ids.contains(id) {
                handle.abort();
                false
            } else {
                true
            }
        });

        // Start monitors for new devices
        for device in devices {
            let id = device.id.expect("Device ID is required");
            if !active_monitors.contains_key(&id) {
                let repo = Arc::clone(&self.repository);
                let app_handle = self.app_handle.clone();
                let handle = tokio::spawn(async move {
                    run_monitor(app_handle, repo, device).await;
                });
                active_monitors.insert(id, handle);
            }
        }

        Ok(())
    }
}

async fn run_monitor(app_handle: AppHandle, repo: Arc<Repository>, device: Device) {
    let mut last_status = true; // Assume online for first state comparison
    let mut state_initialized = false;

    loop {
        // Ping
        let start = std::time::Instant::now();
        let ping_result = perform_ping(&device.ip).await;
        let latency = start.elapsed();

        let (is_online, error) = match ping_result {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e)),
        };

        // Persist check
        let check = Check {
            id: None,
            device_id: device.id.unwrap(),
            timestamp: None,
            is_online,
            latency_ms: Some(latency.as_secs_f64() * 1000.0),
            error_msg: error,
        };

        if let Err(e) = repo.insert_check(&check).await {
            eprintln!("Failed to insert check for device {}: {}", device.id.unwrap(), e);
        }

        // Emit check event
        let _ = app_handle.emit("check-event", &check);

        // Transition logic
        if !state_initialized {
            last_status = is_online;
            state_initialized = true;
        } else if is_online != last_status {
            // Detected transition!
            let transition = crate::models::Transition {
                id: None,
                device_id: device.id.unwrap(),
                from_status: if last_status { "Online".to_string() } else { "Offline".to_string() },
                to_status: if is_online { "Online".to_string() } else { "Offline".to_string() },
                timestamp: None,
            };

            if let Err(e) = repo.insert_transition(&transition).await {
                eprintln!("Failed to insert transition for device {}: {}", device.id.unwrap(), e);
            }

            // Emit transition event
            let _ = app_handle.emit("transition-event", &transition);
            
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



async fn perform_ping(ip: &str) -> Result<(), String> {
    // Placeholder for T011 (ICMP Adapter)
    // For now, simulate success
    Ok(())
}

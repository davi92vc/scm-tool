use crate::repository::Repository;
use std::net::Ipv4Addr;
use std::str::FromStr;

pub struct DeviceService {
    repository: Repository,
}

impl DeviceService {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub async fn create_device(&self, name: &str, ip: &str) -> Result<i64, String> {
        // Rule: Validar IPv4
        if Ipv4Addr::from_str(ip).is_err() {
            return Err("Invalid IPv4 address".to_string());
        }

        // Rule: Bloquear inclusão acima de 4 dispositivos
        let devices = self
            .repository
            .get_all_devices()
            .await
            .map_err(|e| e.to_string())?;

        if devices.len() >= 4 {
            return Err("MVP limit reached (max 4 devices)".to_string());
        }

        // Rule: Bloquear IP duplicado ativo
        if devices.iter().any(|d| d.ip == ip) {
            return Err("IP address already monitored".to_string());
        }

        self.repository
            .create_device(name, ip)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_all_devices(&self) -> Result<Vec<crate::models::Device>, String> {
        self.repository
            .get_all_devices()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_device(&self, id: i64) -> Result<(), String> {
        self.repository
            .delete_device(id)
            .await
            .map_err(|e| e.to_string())
    }
}

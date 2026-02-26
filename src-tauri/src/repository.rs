use crate::models::{AppError, Check, Device, Transition};
use sqlx::SqlitePool;

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // Devices
    pub async fn get_all_devices(&self) -> Result<Vec<Device>, sqlx::Error> {
        sqlx::query_as::<_, Device>("SELECT * FROM devices WHERE is_active = 1")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_device(&self, name: &str, ip: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO devices (name, ip) VALUES (?, ?)")
            .bind(name)
            .bind(ip)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update_device(&self, id: i64, name: &str, ip: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE devices SET name = ?, ip = ? WHERE id = ?")
            .bind(name)
            .bind(ip)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_device(&self, id: i64) -> Result<(), sqlx::Error> {
        // We set is_active to 0 for "soft delete" if we want to keep history?
        // PRD says "CRUD de dispositivos". I'll do hard delete if PRD allows.
        // Wait, "dispositivo online/offline checado a cada 10/2s".
        // I'll do hard delete or soft delete?
        // Actually, PRD says "remover dispositivos".
        sqlx::query("DELETE FROM devices WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Checks
    pub async fn insert_check(&self, check: &Check) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO checks (device_id, is_online, latency_ms, error_msg) VALUES (?, ?, ?, ?)",
        )
        .bind(check.device_id)
        .bind(check.is_online)
        .bind(check.latency_ms)
        .bind(&check.error_msg)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    // Transitions
    pub async fn insert_transition(&self, transition: &Transition) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO transitions (device_id, from_status, to_status) VALUES (?, ?, ?)",
        )
        .bind(transition.device_id)
        .bind(&transition.from_status)
        .bind(&transition.to_status)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    // Errors
    pub async fn insert_error(&self, error: &AppError) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO app_errors (source, message) VALUES (?, ?)")
            .bind(&error.source)
            .bind(&error.message)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn list_app_errors(&self, limit: i64) -> Result<Vec<AppError>, sqlx::Error> {
        sqlx::query_as::<_, AppError>(
            "SELECT * FROM app_errors ORDER BY id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
}

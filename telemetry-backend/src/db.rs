use sqlx::{Pool, Sqlite};

pub async fn init(db: &Pool<Sqlite>) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id_hash TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
        CREATE INDEX IF NOT EXISTS idx_events_device ON events(device_id_hash);
        "#,
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn insert_event(
    db: &Pool<Sqlite>,
    event: &crate::TelemetryEvent,
) -> anyhow::Result<()> {
    let event_type = match &event.event_type {
        crate::EventType::Install { .. } => "install",
        crate::EventType::FirstLaunch { .. } => "first_launch",
        crate::EventType::ShellInit { .. } => "shell_init",
        crate::EventType::FeatureUse { .. } => "feature_use",
        crate::EventType::UpdateCheck { .. } => "update_check",
        crate::EventType::Feedback { .. } => "feedback",
        crate::EventType::Diagnostic { .. } => "diagnostic",
    };

    let payload = serde_json::to_string(&event.event_type)?;

    sqlx::query(
        r#"
        INSERT INTO events (device_id_hash, timestamp, event_type, payload)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(&event.device_id_hash)
    .bind(event.timestamp)
    .bind(event_type)
    .bind(payload)
    .execute(db)
    .await?;

    Ok(())
}

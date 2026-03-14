use sqlx::{Pool, Sqlite};
use crate::models::*;

pub async fn init(db: &Pool<Sqlite>) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feedback (
            id TEXT PRIMARY KEY,
            category TEXT NOT NULL,
            content TEXT NOT NULL,
            contact TEXT,
            version TEXT NOT NULL,
            os TEXT NOT NULL,
            status TEXT DEFAULT 'new',
            response TEXT,
            is_public BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_feedback_status ON feedback(status);
        CREATE INDEX IF NOT EXISTS idx_feedback_created ON feedback(created_at);
        "#,
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn insert_feedback(
    db: &Pool<Sqlite>,
    id: &str,
    feedback: &FeedbackSubmit,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO feedback (id, category, content, contact, version, os)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(id)
    .bind(&feedback.category)
    .bind(&feedback.content)
    .bind(&feedback.contact)
    .bind(&feedback.version)
    .bind(&feedback.os)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_all_feedback(db: &Pool<Sqlite>) -> anyhow::Result<Vec<Feedback>> {
    let feedback = sqlx::query_as!(
        Feedback,
        r#"
        SELECT
            id,
            category,
            content,
            contact,
            version,
            os,
            status,
            created_at,
            response,
            is_public
        FROM feedback
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(db)
    .await?;

    Ok(feedback)
}

pub async fn get_public_feedback_list(db: &Pool<Sqlite>) -> anyhow::Result<Vec<PublicFeedback>> {
    let feedback = sqlx::query_as!(
        PublicFeedback,
        r#"
        SELECT
            id,
            category,
            content,
            status,
            created_at,
            response
        FROM feedback
        WHERE is_public = 1
        ORDER BY created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(db)
    .await?;

    Ok(feedback)
}

pub async fn update_feedback_status(
    db: &Pool<Sqlite>,
    id: &str,
    status: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        UPDATE feedback SET status = ?2 WHERE id = ?1
        "#,
    )
    .bind(id)
    .bind(status)
    .execute(db)
    .await?;

    Ok(())
}

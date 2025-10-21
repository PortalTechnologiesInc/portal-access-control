use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct PublicKey {
    pub id: Uuid,
    pub npub: String,
    pub nip05: Option<String>,
    pub profile_name: Option<String>,
    pub status: bool,
    pub created_at: DateTime<Utc>,
}

// Database helper functions

pub async fn get_all_keys(pool: &Pool<Postgres>) -> Result<Vec<PublicKey>, sqlx::Error> {
    sqlx::query_as::<_, PublicKey>("SELECT * FROM keys ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn insert_key(
    pool: &Pool<Postgres>,
    npub: &str,
    nip05: Option<&str>,
    profile_name: Option<&str>,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO keys (id, npub, nip05, profile_name, status, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(id)
    .bind(npub)
    .bind(nip05)
    .bind(profile_name)
    .bind(true) // Default to enabled
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn toggle_key_status(pool: &Pool<Postgres>, key_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE keys SET status = NOT status WHERE id = $1")
        .bind(key_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_key_by_id(pool: &Pool<Postgres>, key_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM keys WHERE id = $1")
        .bind(key_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn is_key_enabled(pool: &Pool<Postgres>, npub: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, bool>("SELECT status FROM keys WHERE npub = $1")
        .bind(npub)
        .fetch_optional(pool)
        .await?;

    Ok(result.unwrap_or(false))
}

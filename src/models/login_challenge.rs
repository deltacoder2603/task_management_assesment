use chrono::{
    DateTime,
    Utc,
};

use serde::{
    Deserialize,
    Serialize,
};

use sqlx::FromRow;

use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    FromRow,
)]
pub struct LoginChallenge {
    pub id: Uuid,

    pub user_id: Uuid,

    pub code_hash: String,

    pub expires_at: DateTime<Utc>,

    pub used: bool,

    pub created_at: DateTime<Utc>,
}
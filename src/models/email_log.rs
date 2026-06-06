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
pub struct EmailLog {
    pub id: Uuid,

    pub email: String,

    pub code: String,

    pub created_at: DateTime<Utc>,
}
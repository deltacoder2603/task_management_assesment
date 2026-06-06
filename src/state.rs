use dashmap::DashMap;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,

    // user_id -> cached task response
    pub task_cache: Arc<DashMap<String, Value>>,
}
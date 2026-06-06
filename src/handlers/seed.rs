use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
};

use uuid::Uuid;

use crate::{
    auth::password::hash_password,
    state::AppState,
};

pub async fn seed_users(
    State(state): State<Arc<AppState>>,
) -> StatusCode {

    let admin_hash =
        hash_password("admin123")
            .unwrap();

    let james_hash =
        hash_password("bond007")
            .unwrap();

    sqlx::query(
        r#"
        INSERT INTO users
        (
            id,
            full_name,
            email,
            hashed_password,
            role
        )
        VALUES
        ($1,$2,$3,$4,$5)
        ON CONFLICT(email)
        DO NOTHING
        "#
    )
    .bind(Uuid::new_v4())
    .bind("Admin")
    .bind("admin@example.com")
    .bind(admin_hash)
    .bind("admin")
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO users
        (
            id,
            full_name,
            email,
            hashed_password,
            role
        )
        VALUES
        ($1,$2,$3,$4,$5)
        ON CONFLICT(email)
        DO NOTHING
        "#
    )
    .bind(Uuid::new_v4())
    .bind("James Bond")
    .bind("jamesbond@example.com")
    .bind(james_hash)
    .bind("staff")
    .execute(&state.db)
    .await
    .unwrap();

    StatusCode::CREATED
}
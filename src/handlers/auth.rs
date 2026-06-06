use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use chrono::Utc;
use rand::Rng;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::{
        jwt::create_jwt,
        password::{
            hash_password,
            verify_password,
        },
    },
    models::{
        login_challenge::LoginChallenge,
        user::User,
    },
    state::AppState,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub challenge_id: Uuid,
    pub code: String,
}

#[derive(FromRow)]
struct LatestEmail {
    email: String,
    code: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<Value>, StatusCode> {

    let user =
        sqlx::query_as::<_, User>(
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#
        )
        .bind(&payload.email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    verify_password(
        &payload.password,
        &user.hashed_password,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let code = rand::rng()
        .random_range(100000..999999)
        .to_string();

    println!(
        "2FA CODE FOR {} => {}",
        user.email,
        code
    );

    let challenge_id = Uuid::new_v4();

    let code_hash =
        hash_password(&code)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query(
        r#"
        INSERT INTO login_challenges
        (
            id,
            user_id,
            code_hash,
            expires_at,
            used
        )
        VALUES
        (
            $1,
            $2,
            $3,
            NOW() + INTERVAL '5 minutes',
            false
        )
        "#
    )
    .bind(challenge_id)
    .bind(user.id)
    .bind(code_hash)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query(
        r#"
        INSERT INTO email_logs
        (
            id,
            email,
            code
        )
        VALUES
        (
            $1,
            $2,
            $3
        )
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&user.email)
    .bind(&code)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "message": "2FA code sent",
        "login_challenge_id": challenge_id
    })))
}

pub async fn latest_email_log(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {

    let row =
        sqlx::query_as::<_, LatestEmail>(
            r#"
            SELECT
                email,
                code
            FROM email_logs
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "email": row.email,
        "code": row.code
    })))
}

pub async fn verify_2fa(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Verify2FARequest>,
) -> Result<Json<Value>, StatusCode> {

    let challenge =
        sqlx::query_as::<_, LoginChallenge>(
            r#"
            SELECT *
            FROM login_challenges
            WHERE id = $1
            "#
        )
        .bind(payload.challenge_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if challenge.used {
        return Err(StatusCode::BAD_REQUEST);
    }

    if challenge.expires_at < Utc::now() {
        return Err(StatusCode::BAD_REQUEST);
    }

    verify_password(
        &payload.code,
        &challenge.code_hash,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user =
        sqlx::query_as::<_, User>(
            r#"
            SELECT *
            FROM users
            WHERE id = $1
            "#
        )
        .bind(challenge.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    sqlx::query(
        r#"
        UPDATE login_challenges
        SET used = true
        WHERE id = $1
        "#
    )
    .bind(challenge.id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token =
        create_jwt(
            user.id,
            user.email.clone(),
            user.role.clone(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "access_token": token
    })))
}
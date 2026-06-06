use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::jwt::Claims,
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub priority: String,
}

#[derive(Deserialize)]
pub struct AssignTasksRequest {
    pub task_ids: Vec<Uuid>,
    pub assigned_to_email: String,
}

#[derive(Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: String,
}

#[derive(FromRow)]
struct UserLookup {
    id: Uuid,
    email: String,
}

#[derive(FromRow)]
struct AssignedTask {
    id: Uuid,
    title: String,
    status: String,
    priority: String,
    email: String,
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<Value>, StatusCode> {

    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    let task_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO tasks
        (
            id,
            title,
            description,
            status,
            priority,
            created_by_id
        )
        VALUES
        (
            $1,
            $2,
            $3,
            'todo',
            $4,
            $5
        )
        "#
    )
    .bind(task_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.priority)
    .bind(Uuid::parse_str(&claims.sub).unwrap())
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "message": "task created",
        "task_id": task_id
    })))
}

pub async fn assign_tasks(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AssignTasksRequest>,
) -> Result<Json<Value>, StatusCode> {

    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    let user =
        sqlx::query_as::<_, UserLookup>(
            r#"
            SELECT
                id,
                email
            FROM users
            WHERE email = $1
            "#
        )
        .bind(&payload.assigned_to_email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    for task_id in &payload.task_ids {

        sqlx::query(
            r#"
            UPDATE tasks
            SET assigned_to_id = $1
            WHERE id = $2
            "#
        )
        .bind(user.id)
        .bind(task_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    state
        .task_cache
        .remove(&user.id.to_string());

    Ok(Json(json!({
        "message": "tasks assigned",
        "assigned_to": user.email,
        "count": payload.task_ids.len()
    })))
}

pub async fn view_my_tasks(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, StatusCode> {

    if let Some(cached) =
        state.task_cache.get(&claims.sub)
    {
        let mut response =
            cached.clone();

        response["cache"] = json!({
            "hit": true
        });

        return Ok(Json(response));
    }

    let user_id =
        Uuid::parse_str(&claims.sub)
            .unwrap();

    let rows =
        sqlx::query_as::<_, AssignedTask>(
            r#"
            SELECT
                t.id,
                t.title,
                t.status,
                t.priority,
                u.email
            FROM tasks t
            JOIN users u
                ON t.assigned_to_id = u.id
            WHERE t.assigned_to_id = $1
            ORDER BY t.created_at
            "#
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tasks: Vec<Value> =
        rows
            .into_iter()
            .map(|row| {
                json!({
                    "id": row.id,
                    "title": row.title,
                    "status": row.status,
                    "priority": row.priority,
                    "assigned_to": row.email
                })
            })
            .collect();

    let response = json!({
        "user": {
            "email": claims.email,
            "role": claims.role
        },
        "tasks": tasks,
        "summary": {
            "total_assigned_tasks": tasks.len()
        },
        "cache": {
            "hit": false
        }
    });

    state
        .task_cache
        .insert(
            claims.sub.clone(),
            response.clone(),
        );

    Ok(Json(response))
}
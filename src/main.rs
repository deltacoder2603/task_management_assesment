mod state;
mod models;
mod handlers;
mod cache;

mod auth {
    pub mod jwt;
    pub mod password;
    pub mod middleware;
}

use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use dotenvy::dotenv;
use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};

use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL")
            .expect("DATABASE_URL not set");

    let db: PgPool =
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

    let state = Arc::new(AppState {
        db,
        task_cache: Default::default(),
    });

    // Public routes
    let public_routes = Router::new()
        .route(
            "/seed/users",
            post(handlers::seed::seed_users),
        )
        .route(
            "/auth/login",
            post(handlers::auth::login),
        )
        .route(
            "/auth/verify-2fa",
            post(handlers::auth::verify_2fa),
        )
        .route(
            "/dev/email-logs/latest",
            get(handlers::auth::latest_email_log),
        );

    // Protected routes
    let protected_routes = Router::new()
        .route(
            "/tasks",
            post(handlers::tasks::create_task),
        )
        .route(
            "/tasks/assign",
            post(handlers::tasks::assign_tasks),
        )
        .route(
            "/tasks/view-my-tasks",
            get(handlers::tasks::view_my_tasks),
        )
        .layer(
            middleware::from_fn(
                auth::middleware::auth_middleware,
            ),
        );

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state);

    let listener =
        tokio::net::TcpListener::bind(
            "0.0.0.0:3000",
        )
        .await
        .unwrap();

    println!("🚀 Server running on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .unwrap();
}
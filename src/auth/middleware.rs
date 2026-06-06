use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::auth::jwt::{
    verify_jwt,
    Claims,
};

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {

    let auth_header =
        req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(
                StatusCode::UNAUTHORIZED
            )?;

    let token =
        auth_header
            .strip_prefix("Bearer ")
            .ok_or(
                StatusCode::UNAUTHORIZED
            )?;

    let claims =
        verify_jwt(token)
            .map_err(
                |_| StatusCode::UNAUTHORIZED
            )?;

    req.extensions_mut()
        .insert::<Claims>(
            claims
        );

    Ok(
        next.run(req).await
    )
}
use chrono::{
    Duration,
    Utc,
};

use jsonwebtoken::{
    decode,
    encode,
    DecodingKey,
    EncodingKey,
    Header,
    Validation,
};

use serde::{
    Deserialize,
    Serialize,
};

use uuid::Uuid;

const JWT_SECRET: &[u8] =
    b"super_secret_key";

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

pub fn create_jwt(
    user_id: Uuid,
    email: String,
    role: String,
) -> anyhow::Result<String> {

    let expiration =
        Utc::now()
            .checked_add_signed(
                Duration::hours(24),
            )
            .unwrap()
            .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email,
        role,
        exp: expiration as usize,
    };

    let token =
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(
                JWT_SECRET,
            ),
        )?;

    Ok(token)
}

pub fn verify_jwt(
    token: &str,
) -> anyhow::Result<Claims> {

    let token_data =
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(
                JWT_SECRET,
            ),
            &Validation::default(),
        )?;

    Ok(token_data.claims)
}
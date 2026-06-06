use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
    },
    Argon2,
};

pub fn hash_password(
    password: &str,
) -> Result<String, String> {

    let salt =
        SaltString::generate(
            &mut OsRng,
        );

    let hash =
        Argon2::default()
            .hash_password(
                password.as_bytes(),
                &salt,
            )
            .map_err(|e| e.to_string())?
            .to_string();

    Ok(hash)
}

pub fn verify_password(
    password: &str,
    hash: &str,
) -> Result<(), String> {

    let parsed_hash =
        PasswordHash::new(hash)
            .map_err(|e| e.to_string())?;

    Argon2::default()
        .verify_password(
            password.as_bytes(),
            &parsed_hash,
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}
use crate::models::{Claims, User};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde_json::json;
use std::env;

pub static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
});

pub static DECODING_KEY: Lazy<DecodingKey> = Lazy::new(|| {
    DecodingKey::from_secret(JWT_SECRET.as_bytes())
});

pub static ENCODING_KEY: Lazy<EncodingKey> = Lazy::new(|| {
    EncodingKey::from_secret(JWT_SECRET.as_bytes())
});

pub fn create_jwt(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.as_ref().unwrap().to_string(),
        user_id: user.id.as_ref().unwrap().to_string(),
        email: user.email.clone(),
        is_admin: user.is_admin,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &ENCODING_KEY)
}

pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| AuthError::MissingToken)?;

        let token_data = decode::<Claims>(bearer.token(), &DECODING_KEY, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthUser(token_data.claims))
    }
}

pub struct AdminUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        
        if !claims.is_admin {
            return Err(AuthError::Unauthorized);
        }

        Ok(AdminUser(claims))
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    Unauthorized,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authentication token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authentication token"),
            AuthError::Unauthorized => (StatusCode::FORBIDDEN, "Unauthorized access"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
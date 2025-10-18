use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    State,
};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (user identifier)
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
}

impl Claims {
    pub fn new(sub: String) -> Self {
        let now = Utc::now();
        Self {
            sub,
            exp: (now + Duration::hours(24)).timestamp() as usize, // 24 hours
            iat: now.timestamp() as usize,
        }
    }
}

pub struct JWTSecret(String);

impl JWTSecret {
    pub fn new(secret: String) -> Self {
        Self(secret)
    }
    
    pub fn get_secret(&self) -> &str {
        &self.0
    }
}

pub fn create_token(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(&Header::default(), claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &validation)?;
    Ok(token_data.claims)
}

pub struct AuthenticatedUser(pub Claims);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt_secret = req.guard::<&State<JWTSecret>>().await;
        let jwt_secret = match jwt_secret {
            Outcome::Success(secret) => secret,
            _ => return Outcome::Error((Status::InternalServerError, ())),
        };

        let auth_header = req.headers().get_one("Authorization");
        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                &header[7..]
            }
            _ => return Outcome::Error((Status::Unauthorized, ())),
        };

        match validate_token(token, jwt_secret.get_secret()) {
            Ok(claims) => Outcome::Success(AuthenticatedUser(claims)),
            Err(_) => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
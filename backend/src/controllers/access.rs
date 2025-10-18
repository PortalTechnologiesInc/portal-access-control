use rocket::{
    get, post, http::Status, serde::json::Json, State, form::Form, http::CookieJar
};
use sqlx::{Pool, Postgres};
use crate::auth::{Claims, create_token, JWTSecret, AuthenticatedUser, set_auth_cookie, remove_auth_cookie};

#[derive(rocket::form::FromForm)]
pub struct AuthRequest {
    password: String,
}


#[get("/health_check")]
pub fn health_check(_pool_state: &State<Pool<Postgres>>) -> Result<Json<String>, Status> {
    Ok(Json("Ok".to_string()))
}

#[post("/login", data = "<auth_request>")]
pub fn login(
    _pool_state: &State<Pool<Postgres>>, 
    jwt_secret: &State<JWTSecret>,
    cookies: &CookieJar<'_>,
    auth_request: Form<AuthRequest>
) -> Result<Json<serde_json::Value>, Status> {
    dotenvy::dotenv().ok();
    
    let expected_pass = std::env::var("AUTH_PASS")
        .map_err(|_| Status::InternalServerError)?;
    
    if auth_request.password == expected_pass {
        let claims = Claims::new("authenticated_user".to_string());
        let token = create_token(&claims, jwt_secret.get_secret())
            .map_err(|_| Status::InternalServerError)?;
        
        set_auth_cookie(cookies, token.clone());
        
        let response = serde_json::json!({
            "message": "Authentication successful",
            "token": token
        });
        
        Ok(Json(response))
    } else {
        Err(Status::Unauthorized)
    }
}

#[get("/protected")]
pub fn protected_endpoint(
    _pool_state: &State<Pool<Postgres>>,
    user: AuthenticatedUser
) -> Result<Json<serde_json::Value>, Status> {
    let response = serde_json::json!({
        "message": "This is a protected endpoint",
        "user": user.0.sub,
        "authenticated": true
    });
    
    Ok(Json(response))
}

#[post("/logout")]
pub fn logout(
    cookies: &CookieJar<'_>
) -> Result<Json<serde_json::Value>, Status> {
    // Remove the authentication cookie
    remove_auth_cookie(cookies);
    
    let response = serde_json::json!({
        "message": "Logged out successfully"
    });
    
    Ok(Json(response))
}
use rocket::{
    get, post, http::Status, serde::json::Json, State, form::Form, http::CookieJar, response::Redirect
};
use rocket_dyn_templates::{Template, context};
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

#[get("/login")]
pub fn login_page() -> Template {
    Template::render("login", context! {})
}

#[get("/logs")]
pub fn logs_page(user: AuthenticatedUser) -> Template {
    Template::render("logs", context! {
        user: user.0.sub
    })
}

#[post("/login", data = "<auth_request>")]
pub fn login(
    _pool_state: &State<Pool<Postgres>>, 
    jwt_secret: &State<JWTSecret>,
    cookies: &CookieJar<'_>,
    auth_request: Form<AuthRequest>
) -> Result<Redirect, Template> {
    dotenvy::dotenv().ok();
    
    let expected_pass = match std::env::var("AUTH_PASS") {
        Ok(pass) => pass,
        Err(_) => {
            return Err(Template::render("login", context! {
                error: "Server configuration error"
            }));
        }
    };
    
    if auth_request.password == expected_pass {
        let claims = Claims::new("authenticated_user".to_string());
        let token = match create_token(&claims, jwt_secret.get_secret()) {
            Ok(token) => token,
            Err(_) => {
                return Err(Template::render("login", context! {
                    error: "Failed to create authentication token"
                }));
            }
        };
        
        set_auth_cookie(cookies, token);
        Ok(Redirect::to("/logs"))
    } else {
        Err(Template::render("login", context! {
            error: "Invalid password"
        }))
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
) -> Redirect {
    // Remove the authentication cookie
    remove_auth_cookie(cookies);
    
    Redirect::to("/login")
}
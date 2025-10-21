use crate::auth::{
    AuthenticatedUser, Claims, JWTSecret, create_token, remove_auth_cookie, set_auth_cookie,
};
use crate::database::helpers::{get_all_keys, insert_key, toggle_key_status, delete_key_by_id};
use rocket::{
    State, form::Form, get, http::CookieJar, http::Status, post, response::Redirect,
    serde::json::Json,
};
use rocket_dyn_templates::{Template, context};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(rocket::form::FromForm)]
pub struct AuthRequest {
    password: String,
}

#[derive(rocket::form::FromForm)]
pub struct KeyRequest {
    npub: String,
    nip05: Option<String>,
    profile_name: Option<String>,
}

#[get("/health_check")]
pub fn health_check(_pool_state: &State<Pool<Postgres>>) -> Result<Json<String>, Status> {
    Ok(Json("Ok".to_string()))
}

#[get("/login")]
pub fn login_page(user: AuthenticatedUser) -> Template {
    if user.is_authenticated() {
        return Redirect::to("/logs");
    }
    Template::render("login", context! {})
}

#[get("/logs")]
pub fn logs_page(user: AuthenticatedUser) -> Template {
    Template::render(
        "logs",
        context! {
            user: user.0.sub
        },
    )
}

#[post("/login", data = "<auth_request>")]
pub fn login(
    _pool_state: &State<Pool<Postgres>>,
    jwt_secret: &State<JWTSecret>,
    cookies: &CookieJar<'_>,
    auth_request: Form<AuthRequest>,
) -> Result<Redirect, Template> {
    dotenvy::dotenv().ok();

    let expected_pass = match std::env::var("AUTH_PASS") {
        Ok(pass) => pass,
        Err(_) => {
            return Err(Template::render(
                "login",
                context! {
                    error: "Server configuration error"
                },
            ));
        }
    };

    if auth_request.password == expected_pass {
        let claims = Claims::new("authenticated_user".to_string());
        let token = match create_token(&claims, jwt_secret.get_secret()) {
            Ok(token) => token,
            Err(_) => {
                return Err(Template::render(
                    "login",
                    context! {
                        error: "Failed to create authentication token"
                    },
                ));
            }
        };

        set_auth_cookie(cookies, token);
        Ok(Redirect::to("/logs"))
    } else {
        Err(Template::render(
            "login",
            context! {
                error: "Invalid password"
            },
        ))
    }
}

#[get("/protected")]
pub fn protected_endpoint(
    _pool_state: &State<Pool<Postgres>>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, Status> {
    let response = serde_json::json!({
        "message": "This is a protected endpoint",
        "user": user.0.sub,
        "authenticated": true
    });

    Ok(Json(response))
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    // Remove the authentication cookie
    remove_auth_cookie(cookies);

    Redirect::to("/login")
}

// Key Management Endpoints

#[get("/keys")]
pub async fn keys_page(
    pool: &State<Pool<Postgres>>,
    _user: AuthenticatedUser,
) -> Result<Template, Template> {
    match get_all_keys(pool).await {
        Ok(keys) => Ok(Template::render(
            "keys",
            context! {
                keys: keys
            },
        )),
        Err(e) => {
            dbg!(e);
            Err(Template::render(
                "keys",
                context! {
                    error_message: "Failed to load keys"
                },
            ))
        }
    }
}

#[post("/keys", data = "<key_request>")]
pub async fn add_key(
    pool: &State<Pool<Postgres>>,
    _user: AuthenticatedUser,
    key_request: Form<KeyRequest>,
) -> Result<Redirect, Template> {
    // Validate npub format
    if !key_request.npub.starts_with("npub1") || key_request.npub.len() != 63 {
        return Err(render_keys_with_error(pool, "Invalid public key format. Must be a valid npub1 key.").await);
    }

    match insert_key(
        pool,
        &key_request.npub,
        key_request.nip05.as_deref(),
        key_request.profile_name.as_deref(),
    )
    .await
    {
        Ok(_) => Ok(Redirect::to("/keys")),
        Err(_) => Err(render_keys_with_error(pool, "Failed to add key. It may already exist.").await),
    }
}

#[post("/keys/<key_id>/toggle")]
pub async fn toggle_key(
    pool: &State<Pool<Postgres>>,
    _user: AuthenticatedUser,
    key_id: String,
) -> Result<Redirect, Template> {
    let uuid = match Uuid::parse_str(&key_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err(render_keys_with_error(pool, "Invalid key ID").await);
        }
    };

    match toggle_key_status(pool, uuid).await {
        Ok(_) => Ok(Redirect::to("/keys")),
        Err(_) => Err(render_keys_with_error(pool, "Failed to toggle key status").await),
    }
}

#[post("/keys/<key_id>/delete")]
pub async fn delete_key(
    pool: &State<Pool<Postgres>>,
    _user: AuthenticatedUser,
    key_id: String,
) -> Result<Redirect, Template> {
    let uuid = match Uuid::parse_str(&key_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err(render_keys_with_error(pool, "Invalid key ID").await);
        }
    };

    match delete_key_by_id(pool, uuid).await {
        Ok(_) => Ok(Redirect::to("/keys")),
        Err(_) => Err(render_keys_with_error(pool, "Failed to delete key").await),
    }
}

// Helper function to render keys template with error message
async fn render_keys_with_error(
    pool: &Pool<Postgres>,
    error_message: &str,
) -> Template {
    match get_all_keys(pool).await {
        Ok(keys) => Template::render(
            "keys",
            context! {
                keys: keys,
                error_message: error_message
            },
        ),
        Err(_) => Template::render(
            "keys",
            context! {
                error_message: error_message
            },
        ),
    }
}

#[catch(401)]
pub fn unauthorized_handler(_req: &Request) -> Redirect {
    Redirect::to("/login")
}

#[catch(404)]
pub fn not_found_handler(_req: &Request) -> Redirect {
    Redirect::to("/logs")
}
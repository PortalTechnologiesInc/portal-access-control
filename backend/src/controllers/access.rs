use crate::auth::{
    AuthenticatedUser, Claims, JWTSecret, create_token, remove_auth_cookie, set_auth_cookie,
};
use chrono::{DateTime, Utc};
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

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct PublicKey {
    pub id: Uuid,
    pub npub: String,
    pub nip05: Option<String>,
    pub profile_name: Option<String>,
    pub status: bool,
    pub created_at: DateTime<Utc>,
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
        return Err(Template::render(
            "keys",
            context! {
                error_message: "Invalid public key format. Must be a valid npub1 key."
            },
        ));
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
        Err(_) => Err(Template::render(
            "keys",
            context! {
                error_message: "Failed to add key. It may already exist."
            },
        )),
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
            return Err(Template::render(
                "keys",
                context! {
                    error_message: "Invalid key ID"
                },
            ));
        }
    };

    match toggle_key_status(pool, uuid).await {
        Ok(_) => Ok(Redirect::to("/keys")),
        Err(_) => Err(Template::render(
            "keys",
            context! {
                error_message: "Failed to toggle key status"
            },
        )),
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
            return Err(Template::render(
                "keys",
                context! {
                    error_message: "Invalid key ID"
                },
            ));
        }
    };

    match delete_key_by_id(pool, uuid).await {
        Ok(_) => Ok(Redirect::to("/keys")),
        Err(_) => Err(Template::render(
            "keys",
            context! {
                error_message: "Failed to delete key"
            },
        )),
    }
}

// Database helper functions

async fn get_all_keys(pool: &Pool<Postgres>) -> Result<Vec<PublicKey>, sqlx::Error> {
    sqlx::query_as::<_, PublicKey>("SELECT * FROM keys ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

async fn insert_key(
    pool: &Pool<Postgres>,
    npub: &str,
    nip05: Option<&str>,
    profile_name: Option<&str>,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO keys (id, npub, nip05, profile_name, status, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(id)
    .bind(npub)
    .bind(nip05)
    .bind(profile_name)
    .bind(true) // Default to enabled
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

async fn toggle_key_status(pool: &Pool<Postgres>, key_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE keys SET status = NOT status WHERE id = $1")
        .bind(key_id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn delete_key_by_id(pool: &Pool<Postgres>, key_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM keys WHERE id = $1")
        .bind(key_id)
        .execute(pool)
        .await?;

    Ok(())
}

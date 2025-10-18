mod auth;
mod controllers;

use anyhow::Result;
use dotenvy::dotenv;
use rocket::fs::{FileServer, relative};
use rocket::{Build, Rocket, routes};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket_dyn_templates::Template;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::auth::JWTSecret;
use crate::controllers::access::{
    health_check, login, login_page, logout, logs_page, protected_endpoint,
};

async fn db_setup() -> Result<Pool<Postgres>> {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create connection pool
    let pool = PgPoolOptions::new().connect(&db_url).await?;
    Ok(pool)
}

fn build_rocket(pool: Pool<Postgres>) -> Rocket<Build> {
    // Load environment variables
    dotenv().ok();
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![
                rocket::http::Method::Get,
                rocket::http::Method::Post,
                rocket::http::Method::Options,
            ]
            .into_iter()
            .map(From::from)
            .collect(),
        )
        .allowed_headers(AllowedHeaders::some(&[
            "Content-Type",
            "Accept",
            "User-Agent",
        ]))
        .allow_credentials(true)
        .max_age(Some(86400)) // 24 hours
        .to_cors()
        .expect("Error creating CORS fairing");

    rocket::build()
        .configure(rocket::Config::figment().merge(("secret_key", jwt_secret.as_bytes())))
        .manage(pool)
        .manage(JWTSecret::new(jwt_secret))
        .mount(
            "/",
            routes![
                health_check,
                login_page,
                login,
                logs_page,
                protected_endpoint,
                logout
            ],
        )
        .mount("/static", FileServer::from(relative!("static")))
        .attach(cors)
        .attach(Template::fairing())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // print_event_for_debug().await;
    let pool = db_setup().await.expect("Database failed to connect");
    build_rocket(pool).launch().await?;

    Ok(())
}

mod controllers;
mod auth;

use anyhow::Result;
use dotenvy::dotenv;
use rocket::{routes, Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::controllers::access::{health_check, login, protected_endpoint};
use crate::auth::JWTSecret;

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
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");
    
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![rocket::http::Method::Get, rocket::http::Method::Post, rocket::http::Method::Options]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allowed_headers(AllowedHeaders::some(&[
            "Content-Type",
            "Accept",
            "User-Agent",
            "Authorization",
        ]))
        .allow_credentials(false)
        .max_age(Some(86400)) // 24 hours
        .to_cors()
        .expect("Error creating CORS fairing");

    rocket::build()
        .manage(pool)
        .manage(JWTSecret::new(jwt_secret))
        .mount("/", routes![health_check, login, protected_endpoint])
        .attach(cors)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // print_event_for_debug().await;
    let pool = db_setup().await.expect("Database failed to connect");
    build_rocket(pool).launch().await?;

    Ok(())
}
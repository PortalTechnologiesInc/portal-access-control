mod auth;
mod controllers;
mod database;

use anyhow::Result;
use dotenvy::dotenv;
use portal::nostr::nips::nip19::ToBech32;
use rocket::fs::{FileServer, relative};
use rocket::tokio::sync::Mutex;
use rocket::{Build, Rocket, routes};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket_dyn_templates::Template;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::env;
use std::sync::Arc;

use crate::auth::JWTSecret;
use crate::controllers::access::{
    add_key, delete_key, health_check, keys_page, login, login_page, logout, logs_page,
    protected_endpoint, toggle_key,
};
use crate::database::helpers::is_key_enabled;

use access_control::DoorUnlockClient;
use portal::protocol::model::auth::AuthResponseStatus;

async fn db_setup() -> Result<Pool<Postgres>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

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
                logout,
                keys_page,
                add_key,
                toggle_key,
                delete_key
            ],
        )
        .mount("/static", FileServer::from(relative!("static")))
        .attach(cors)
        .attach(Template::fairing())
}

async fn build_access_ontrol(pool: Pool<Postgres>) {
    // Read configuration from environment variables
    let base_url =
        env::var("INTELLIM_BASE_URL").expect("INTELLIM_BASE_URL environment variable is required");

    let username =
        env::var("INTELLIM_USERNAME").expect("INTELLIM_USERNAME environment variable is required");

    let password =
        env::var("INTELLIM_PASSWORD").expect("INTELLIM_PASSWORD environment variable is required");

    // Portal configuration
    let nostr_key =
        env::var("PORTAL_NOSTR_KEY").expect("PORTAL_NOSTR_KEY environment variable is required");

    let relay_url =
        env::var("PORTAL_RELAY_URL").expect("PORTAL_RELAY_URL environment variable is required");

    let door_id = env::var("DOOR_ID")
        .expect("DOOR_ID environment variable is required")
        .parse::<u32>()
        .expect("DOOR_ID must be a valid number");

    println!("=== IntelliM Door Access Control Client (Rocket) ===");
    println!("Connecting to: {}", base_url);
    println!("Username: {}", username);
    println!("Door ID: {}", door_id);

    // Initialize the door unlock client and Portal SDK
    let client = Arc::new(Mutex::new(DoorUnlockClient::new(
        base_url.clone(),
        username,
        password,
    )));

    // Portal SDK initialization (may return Result; unwrap/expect for simplicity)
    let keys = portal::nostr::Keys::parse(&nostr_key).expect("Failed to parse nostr key");
    let keypair = portal::protocol::LocalKeypair::new(keys, None);
    let portal_sdk = Arc::new(
        sdk::PortalSDK::new(keypair, vec![relay_url])
            .await
            .expect("Failed to initialize Portal SDK"),
    );

    // Clone Arcs for the background task
    let bg_client = Arc::clone(&client);
    let bg_portal = Arc::clone(&portal_sdk);

    // Spawn the long-running handshake/notification loop as a background task on the Rocket/Tokio runtime.
    // DO NOT create another tokio runtime. Use rocket::tokio::spawn (or tokio::spawn) instead.
    rocket::tokio::spawn(async move {
        println!("Portal SDK background task started. Waiting for authentication requests...");
        loop {
            // Create a handshake URL and receive a notifications stream
            match bg_portal
                .new_key_handshake_url(Some("1910-main-cafe-entrance".to_string()), Some(false))
                .await
            {
                Ok((key_handshake_url, mut notifications)) => {
                    println!("Key handshake URL: {}", key_handshake_url);

                    // Process notification stream until it ends or errors out
                    while let Some(notification_result) = notifications.next().await {
                        match notification_result {
                            Err(e) => {
                                println!("❌ Notification error: {:?}", e);
                                // continue to wait for next notification or recreate handshake if stream ended
                                continue;
                            }
                            Ok(event) => {
                                let npub = event.main_key;
                                
                                match is_key_enabled(&pool, npub.to_bech32().expect("Infallible").as_str()).await {
                                    Ok(true) => {
                                        println!("✅ Key is enabled, proceeding with authentication");
                                    }
                                    Ok(false) => {
                                        println!("❌ Key is disabled, skipping authentication");
                                        continue;
                                    }
                                    Err(e) => {
                                        // Database error - log and skip
                                        println!("❌ Database error checking key: {:?}", e);
                                        continue;
                                    }
                                }

                                // Authenticate the key obtained from the notification
                                match bg_portal.authenticate_key(npub, vec![]).await {
                                    Ok(response) => {
                                        match response.status {
                                            AuthResponseStatus::Approved { .. } => {
                                                println!("✅ Authentication successful");
                                                // Attempt to unlock the door
                                                match bg_client
                                                    .lock()
                                                    .await
                                                    .unlock_door(door_id, Some(-1))
                                                    .await
                                                {
                                                    Ok(unlock_response) => {
                                                        if unlock_response.success {
                                                            println!(
                                                                "✅ Door {} unlocked successfully",
                                                                door_id
                                                            );
                                                        } else {
                                                            println!(
                                                                "❌ Door unlock failed: {}",
                                                                unlock_response.message
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        println!("❌ Door unlock error: {}", e);
                                                    }
                                                }
                                            }
                                            AuthResponseStatus::Declined { .. } => {
                                                println!("❌ Authentication declined");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("❌ Authentication error: {:?}", e);
                                    }
                                }
                            }
                        }
                    }

                    // If we get here the notification stream ended. Loop will recreate a new handshake URL.
                    println!("Notification stream ended, re-creating handshake URL...");
                }
                Err(e) => {
                    // Creating handshake URL failed; back off a bit and retry.
                    println!("❌ Failed to create handshake URL: {:?}", e);
                    rocket::tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    });
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // print_event_for_debug().await;
    let pool = db_setup().await.expect("Database failed to connect");
    build_access_ontrol(pool.clone()).await;
    build_rocket(pool).launch().await?;

    Ok(())
}

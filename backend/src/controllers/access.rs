use rocket::{
    get, http::Status, serde::json::Json, State
};
use sqlx::{Pool, Postgres};


#[get("/health_check")]
pub fn health_check(_pool_state: &State<Pool<Postgres>>) -> Result<Json<String>, Status> {
    Ok(Json("Ok".to_string()))
}

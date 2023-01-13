mod account;
mod data;
mod session;

use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    extract::{FromRef, State},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};

// Its data should be before Arc in order to safely clone to middleware.
#[derive(Clone)]
pub struct AppData {
    database_pool: Arc<PgPool>,
}

impl FromRef<AppData> for Arc<PgPool> {
    fn from_ref(input: &AppData) -> Self {
        input.database_pool.clone()
    }
}

impl AppData {
    pub fn new(pool: PgPool) -> Self {
        Self {
            database_pool: Arc::new(pool),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().unwrap();
    tracing_subscriber::fmt().init();

    let database_url =
        std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL env variable.");

    let pg_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .context("Unable to connect to Postgres database.")?;

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .context("Unable to perform database migration.")?;

    let app_data = AppData::new(pg_pool);

    let router = Router::new()
        .nest("/users", account::users_router())
        .nest("/session", session::session_router())
        .nest("/data", data::data_router())
        .layer(axum::middleware::from_fn_with_state(
            app_data.clone(),
            session::session_management,
        ))
        .with_state(app_data)
        .layer(
            tower_http::cors::CorsLayer::very_permissive()
                //.allow_origin("http://127.0.0.1/".parse::<http::HeaderValue>().unwrap())
                .allow_credentials(true),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let address = "127.0.0.1:47123".parse().unwrap();

    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}

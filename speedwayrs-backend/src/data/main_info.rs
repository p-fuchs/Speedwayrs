use axum::{extract::State, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use http::StatusCode;
use serde::{Serialize, Deserialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use std::sync::Arc;

use crate::session::SESSION_COOKIE;

#[derive(Serialize, FromRow)]
struct ResponseForm {
    game_id: i32,
    team1: String,
    team1_id: i32,
    team2: String,
    team2_id: i32,
    date: time::OffsetDateTime,
    score: Option<String>
}

#[derive(Deserialize)]
pub struct RequestInfo {
    page: i64 
}

const PAGE_SIZE: i64 = 5;

pub async fn last_games(State(db): State<Arc<PgPool>>, Json(info): Json<RequestInfo>) -> impl IntoResponse {
    let query = sqlx::query_file_as!(ResponseForm, "queries/data/get_last_games.sql", PAGE_SIZE, (info.page - 1) * PAGE_SIZE)
        .fetch_all(db.as_ref())
        .await;

    match query {
        Err(e) => {
            tracing::error!("Error returned while querying last_games. Error = [{e:?}]");

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
        Ok(info) => {
            (StatusCode::OK, Json(info)).into_response()
        }
    }
}

pub async fn liked_teams(State(db): State<Arc<PgPool>>, jar: CookieJar) -> impl IntoResponse {
    let session_id = Uuid::parse_str(jar.get(SESSION_COOKIE).expect("Cannot find session cookie.").value()).expect("Middleware does not work as expected.");

    let query = sqlx::query_file_as!(ResponseForm, "queries/data/get_liked_teams.sql", session_id, PAGE_SIZE)
        .fetch_all(db.as_ref())
        .await;

    match query {
        Err(e) => {
            tracing::error!("Error returned while querying liked_teams. Error = [{e:?}]");

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
        Ok(rows) => {
            return (StatusCode::OK, Json(rows)).into_response()
        }
    }
}

pub async fn liked_players(State(db): State<Arc<PgPool>>, jar: CookieJar) -> impl IntoResponse {
    let session_id = Uuid::parse_str(jar.get(SESSION_COOKIE).expect("Cannot find session cookie.").value()).expect("Middleware does not work as expected.");

    let query = sqlx::query_file_as!(ResponseForm, "queries/data/get_liked_players.sql", session_id, PAGE_SIZE)
        .fetch_all(db.as_ref())
        .await;

    match query {
        Err(e) => {
            tracing::error!("Error returned while querying liked_players. Error = [{e:?}]");

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
        Ok(rows) => {
            (StatusCode::OK, Json(rows)).into_response()
        }
    }
}

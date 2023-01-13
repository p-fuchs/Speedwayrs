use std::sync::Arc;

use axum::{response::IntoResponse, extract::State, body::Body, Json, Router, routing::post};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::AppData;

#[derive(Deserialize)]
struct TeamSearch {
    team_name: String
}

#[derive(Serialize)]
struct Team {
    name: String,
    id: i32
}

async fn search_teams(
    State(db): State<Arc<PgPool>>,
    Json(form): Json<TeamSearch>
) -> impl IntoResponse {
    let name = form.team_name;

    let query: sqlx::Result<Vec<_>> = sqlx::query_file!("queries/team_search.sql", format!("%{name}%"))
        .fetch_all(db.as_ref())
        .await;
    
    match query {
        Ok(username) => {
            let teams: Vec<Team> = username.into_iter().map(|record| {
                Team {
                    name: record.team_name,
                    id: record.team_id
                }
            }).collect();

            (StatusCode::OK, Json(teams)).into_response()
        }
        Err(e) => {
            tracing::error!("Error while requesting team_search from database: {e:?}");

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

pub fn data_router() -> Router<AppData> {
    Router::new()
        .route("/teams", post(search_teams))
}
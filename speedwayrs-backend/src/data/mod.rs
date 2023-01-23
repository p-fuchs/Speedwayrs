mod player_data;
mod main_info;
mod match_info;
mod team_data;
mod team_stats;

use std::sync::Arc;

use axum::{body::Body, extract::State, response::IntoResponse, routing::{post, get}, Json, Router};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::AppData;

#[derive(Deserialize)]
struct TeamSearch {
    team_name: String,
}

#[derive(Deserialize)]
struct PlayerSearch {
    player_name: String,
}

#[derive(Serialize)]
struct Team {
    name: String,
    id: i32,
}

#[derive(Serialize)]
struct Player {
    name: String,
    sname: String,
    id: i32,
}

async fn search_players(
    State(db): State<Arc<PgPool>>,
    Json(form): Json<PlayerSearch>,
) -> impl IntoResponse {
    let name = form.player_name;

    let query: sqlx::Result<Vec<_>> =
        sqlx::query_file!("queries/player_search.sql", format!("%{name}%"))
            .fetch_all(db.as_ref())
            .await;

    match query {
        Err(e) => {
            tracing::error!(
                "Error occured while querying database in search_players(). Error = [{:?}]",
                e
            );

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
        Ok(players) => (
            StatusCode::OK,
            Json(
                players
                    .into_iter()
                    .map(|record| Player {
                        name: record.name,
                        sname: record.sname,
                        id: record.player_id,
                    })
                    .collect::<Vec<Player>>(),
            ),
        )
            .into_response(),
    }
}

async fn search_teams(
    State(db): State<Arc<PgPool>>,
    Json(form): Json<TeamSearch>,
) -> impl IntoResponse {
    let name = form.team_name;

    let query: sqlx::Result<Vec<_>> =
        sqlx::query_file!("queries/team_search.sql", format!("%{name}%"))
            .fetch_all(db.as_ref())
            .await;

    match query {
        Ok(username) => {
            let teams: Vec<Team> = username
                .into_iter()
                .map(|record| Team {
                    name: record.team_name,
                    id: record.team_id,
                })
                .collect();

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
        .route("/players", post(search_players))
        .route("/team_info", post(team_data::team_data))
        .route("/team_stats", post(team_stats::team_stats))
        .route("/match_info", post(match_info::match_info_handler))
        .route("/last_games", post(main_info::last_games))
        .route("/liked_teams", get(main_info::liked_teams))
        .route("/liked_players", get(main_info::liked_players))
        .route("/player_info", post(player_data::get_player_data))
}

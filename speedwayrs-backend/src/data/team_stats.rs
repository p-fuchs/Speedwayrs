use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json, Router};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::AppData;

#[derive(Deserialize)]
pub struct TeamData {
    team_id: i32,
}

#[derive(Serialize)]
struct GameData {
    id: i32,
    opponent: String,
    games: u32,
}

impl GameData {
    pub fn new(id: i32, opponent: String, games: u32) -> Self {
        Self {
            id,
            opponent,
            games,
        }
    }
}

#[derive(Serialize)]
struct TeamInfo {
    wins: u32,
    looses: u32,
    ties: u32,
    often_looses: Vec<GameData>,
    often_wins: Vec<GameData>,
}

const STATS_LIMIT: i64 = 5;

async fn get_totals(team_id: i32, db: &Arc<PgPool>) -> Result<(u32, u32, u32), sqlx::Error> {
    let query = sqlx::query_file!("queries/team_winratio.sql", team_id)
        .fetch_one(db.as_ref())
        .await?;

    let wins = query.wins.unwrap() as u32;
    let total = query.total.unwrap() as u32;
    let ties = query.ties.unwrap() as u32;

    tracing::info!("From get_totals(): wins = {wins}, total = {total}, ties = {ties}");

    Ok((wins, total - wins - ties, ties))
}

async fn get_often_looses(team_id: i32, db: &Arc<PgPool>) -> Result<Vec<GameData>, sqlx::Error> {
    let query = sqlx::query_file!("queries/team_looses.sql", team_id, STATS_LIMIT)
        .fetch_all(db.as_ref())
        .await?;

    Ok(query
        .into_iter()
        .map(|record| {
            GameData::new(
                record.team_2.unwrap(),
                record.team_2_name.unwrap(),
                record.looses.unwrap() as u32,
            )
        })
        .collect())
}

async fn get_often_wins(team_id: i32, db: &Arc<PgPool>) -> Result<Vec<GameData>, sqlx::Error> {
    let query = sqlx::query_file!("queries/team_wins.sql", team_id, STATS_LIMIT)
        .fetch_all(db.as_ref())
        .await?;

    Ok(query
        .into_iter()
        .map(|record| {
            GameData::new(
                record.team_2.unwrap(),
                record.team_2_name.unwrap(),
                record.wins.unwrap() as u32,
            )
        })
        .collect())
}

pub async fn team_stats(
    State(db): State<Arc<PgPool>>,
    Json(req_data): Json<TeamData>,
) -> impl IntoResponse {
    let (wins, looses, ties) = match get_totals(req_data.team_id, &db).await {
        Err(e) => {
            tracing::error!(
                "Error returned from database while trying to get_totals(). Error = [{e:?}]"
            );

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
        Ok(res) => res,
    };

    let often_looses = match get_often_looses(req_data.team_id, &db).await {
        Err(e) => {
            tracing::error!(
                "Error returned from database while trying to get often looses. Error = [{e:?}]"
            );

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
        Ok(vec) => vec,
    };

    let often_wins = match get_often_wins(req_data.team_id, &db).await {
        Err(e) => {
            tracing::error!(
                "Error returned from database while trying to get often wins. Error = [{e:?}]"
            );

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
        Ok(vec) => vec,
    };

    let team_data_response = TeamInfo {
        wins,
        looses,
        ties,
        often_looses,
        often_wins,
    };

    (StatusCode::OK, Json(team_data_response)).into_response()
}

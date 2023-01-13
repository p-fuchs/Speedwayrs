use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct TeamInfo {
    team_id: i32,
    skip_first: u16,
    step: u16,
}

#[derive(Serialize)]
struct MatchInfo {
    match_id: i32,
    opponent_name: String,
    opponent_id: i32,
}

#[derive(Serialize)]
struct TeamData {
    team_name: String,
    last_matches: Vec<MatchInfo>,
}

async fn get_team_name(db: &Arc<PgPool>, team_id: i32) -> Result<Option<String>, sqlx::Error> {
    let query_result = sqlx::query_file!("queries/data/team_data_name.sql", team_id)
        .fetch_optional(db.as_ref())
        .await?;

    match query_result {
        None => Ok(None),
        Some(record) => Ok(Some(record.team_name)),
    }
}

async fn get_team_matches(db: &Arc<PgPool>, team_id: i32) -> Result<Vec<MatchInfo>, sqlx::Error> {
    let query_result = sqlx::query_file!("queries/data/team_data_matches.sql", team_id)
        .fetch_all(db.as_ref())
        .await?;

    Ok(query_result
        .into_iter()
        .map(|record| MatchInfo {
            match_id: record.game_id,
            opponent_name: record.opponent.unwrap(),
            opponent_id: record.opponent_id.unwrap(),
        })
        .collect())
}

// We will return team name, last matches
pub(super) async fn team_data(
    State(db): State<Arc<PgPool>>,
    Json(info): Json<TeamInfo>,
) -> impl IntoResponse {
    let team_name;

    match get_team_name(&db, info.team_id).await {
        Ok(None) => {
            return (StatusCode::NOT_FOUND).into_response();
        }
        Ok(Some(name)) => {
            team_name = name;
        }
        Err(e) => {
            tracing::error!(
                "Error occured while querying database from team_data. Error = [{e:?}]"
            );

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    }

    match get_team_matches(&db, info.team_id).await {
        Ok(matches) => {
            let team_data = TeamData {
                team_name,
                last_matches: matches,
            };

            (StatusCode::OK, Json(team_data)).into_response()
        }
        Err(e) => {
            tracing::error!(
                "Error occured while querying database from team_data. Error = [{e:?}]"
            );

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Extension, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::session::AuthStatus;

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
    date: String,
}

#[derive(Serialize)]
struct TeamData {
    team_name: String,
    last_matches: Vec<MatchInfo>,
    user_like: Option<bool>,
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

async fn get_team_matches(
    db: &Arc<PgPool>,
    team_info: &TeamInfo,
) -> Result<Vec<MatchInfo>, sqlx::Error> {
    let query_result = sqlx::query_file!(
        "queries/data/team_data_matches.sql",
        team_info.team_id,
        team_info.skip_first as i32,
        team_info.step as i32
    )
    .fetch_all(db.as_ref())
    .await?;

    Ok(query_result
        .into_iter()
        .map(|record| MatchInfo {
            match_id: record.game_id,
            opponent_name: record.opponent.unwrap(),
            opponent_id: record.opponent_id.unwrap(),
            date: record.game_date.date().to_string(),
        })
        .collect())
}

async fn check_team_like(
    db: &Arc<PgPool>,
    username: &str,
    team_id: i32,
) -> Result<bool, sqlx::Error> {
    match sqlx::query_file!("queries/data/team_data_like.sql", username, team_id)
        .fetch_optional(db.as_ref())
        .await?
    {
        None => Ok(false),
        Some(_) => Ok(true),
    }
}

// We will return team name, last matches
pub(super) async fn team_data(
    State(db): State<Arc<PgPool>>,
    Extension(auth_status): Extension<Arc<AuthStatus>>,
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

    let user_like;

    if let AuthStatus::Authenticated(user) = auth_status.as_ref() {
        match check_team_like(&db, &user, info.team_id).await {
            Err(e) => {
                tracing::error!(
                    "Error occured while querying database from team_data. Error = [{e:?}]"
                );

                return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
            Ok(val) => user_like = Some(val),
        }
    } else {
        println!("AUTH STATUS = {auth_status:?}");
        user_like = None;
    }

    match get_team_matches(&db, &info).await {
        Ok(matches) => {
            let team_data = TeamData {
                team_name,
                last_matches: matches,
                user_like,
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

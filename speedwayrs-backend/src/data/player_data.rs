use axum::{extract::State, response::IntoResponse, Json};
use http::StatusCode;
use serde::{Serialize, Deserialize};
use speedwayrs_types::PlayerResult;
use sqlx::PgPool;
use std::sync::Arc;
use axum::Extension;
use crate::session::AuthStatus;

#[derive(Deserialize)]
pub struct GetPlayerInfo {
    player: i32
}

#[derive(Serialize)]
struct PlayerInfo {
    three_points: u32,
    two_points: u32,
    one_points: u32,
    zero_points: u32,
    stars: u32,
    accidents: u16,
    former_teams: Vec<(i32, String, u16)>,
    name: String,
    user_like: Option<bool>
}

struct PlayerStats {
    three_points: u32,
    two_points: u32,
    one_points: u32,
    zero_points: u32,
    stars: u32,
    accidents: u16
}

async fn check_player_like(db: &PgPool, username: &str, player_id: i32) -> Result<bool, sqlx::Error> {
    let query = sqlx::query_file!("queries/utils/check_like_player.sql", username, player_id)
        .fetch_optional(db)
        .await?;

    Ok(query.is_some())
}

async fn get_player_stats(db: &PgPool, id: i32) -> Result<PlayerStats, sqlx::Error> {
    let query_result = sqlx::query_file!("queries/data/get_player_stats.sql", id)
        .fetch_all(db)
        .await?;

    let mut points = [0 ; 4];
    let mut stars = 0;
    let mut accidents = 0;

    for record in query_result {
        match PlayerResult::from_str(&record.result) {
            Some(player_score) => {
                match player_score {
                    PlayerResult::Score(score) => {
                        points[score as usize] += 1;
                    }
                    PlayerResult::ScoreWithStar(score) => {
                        points[score as usize] += 1;
                        stars += 1;
                    }
                    _ => {
                        accidents += 1;
                    }
                }
            }
            None => {
                tracing::error!("Error while parsing PlayerResult. Value = [{}]", record.result);
            }
        }
    }

    Ok(
        PlayerStats {
            three_points: points[3],
            two_points: points[2],
            one_points: points[1],
            zero_points: points[0],
            stars,
            accidents
        }
    )
}

async fn get_former_teams(db: &PgPool, id: i32) -> Result<Vec<(i32, String, u16)>, sqlx::Error> {
    let former_teams = sqlx::query_file!("queries/data/get_player_teams.sql", id)
        .fetch_all(db)
        .await?;
    
    Ok(former_teams.into_iter().map(|record| (record.team_id, record.team_name, record.game_count.unwrap() as u16)).collect())
}

async fn get_player_name(db: &PgPool, id: i32) -> Result<Option<String>, sqlx::Error> {
    let query = sqlx::query_file!("queries/data/get_player_name.sql", id)
        .fetch_one(db)
        .await?;

    Ok(query.name)
}

pub async fn get_player_data(State(db): State<Arc<PgPool>>, Extension(auth_info): Extension<Arc<AuthStatus>>, Json(info): Json<GetPlayerInfo>) -> impl IntoResponse {
    let player_like = match auth_info.as_ref() {
        AuthStatus::Authenticated(user) => {
            match check_player_like(db.as_ref(), &user, info.player).await {
                Ok(like) => Some(like),
                Err(e) => {
                    tracing::error!("Error returned while querying database about player like. Error = [{e:?}]");

                    return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }
            }
        }
        AuthStatus::NonAuthenticated => {
            None
        }
    };

    let player_name = match get_player_name(db.as_ref(), info.player).await {
        Ok(None) => {
            return (StatusCode::NOT_FOUND).into_response();
        }
        Ok(Some(name)) => name,
        Err(e) => {
            tracing::error!("Error returned while querying player name. Error = [{e:?}]");

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    };

    let player_stats = match get_player_stats(db.as_ref(), info.player).await {
        Err(e) => {
            tracing::error!("Error returned while getting player stats. Error = [{e:?}]");
            
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
        Ok(stats) => stats
    };

    let former_teams = match get_former_teams(db.as_ref(), info.player).await {
        Err(e) => {
            tracing::error!("Error returned while getting former teams. Error = [{e:?}]");

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
        Ok(teams) => teams
    };

    let json_response = PlayerInfo {
        three_points: player_stats.three_points,
        two_points: player_stats.two_points,
        one_points: player_stats.one_points,
        zero_points: player_stats.zero_points,
        stars: player_stats.stars,
        accidents: player_stats.accidents,
        former_teams,
        name: player_name,
        user_like: player_like 
    };

    (StatusCode::OK, Json(json_response)).into_response()
}

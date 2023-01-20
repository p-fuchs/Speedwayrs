use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use http::StatusCode;
use speedwayrs_types::{MatchResult, Player, PlayerResult, RunInfo};
use sqlx::{FromRow, PgPool};

use serde::Deserialize;

#[derive(FromRow)]
struct MatchMainInfo {
    game_id: i32,
    team_1: String,
    score_1: i32,
    score_2: i32,
    team_2: String,
    place: String,
    game_date: time::OffsetDateTime,
}

#[derive(Deserialize)]
pub struct HandlerInfo {
    match_id: i32,
}

async fn get_team_name(
    team_1: i32,
    team_2: i32,
    db: &Arc<PgPool>,
) -> Result<(String, String), sqlx::Error> {
    let query_result = sqlx::query_file!("queries/data/match_info_team_name.sql", team_1, team_2)
        .fetch_all(db.as_ref())
        .await?;

    let mut iterator = query_result.into_iter();

    let team_1_bis = iterator.next().unwrap();
    let team_2_bis = iterator.next().unwrap();

    if team_1_bis.team_id == team_1 {
        Ok((team_1_bis.team_name, team_2_bis.team_name))
    } else {
        Ok((team_2_bis.team_name, team_1_bis.team_name))
    }
}

async fn select_main_data(
    game_id: i32,
    db: &Arc<PgPool>,
) -> Result<Option<MatchMainInfo>, sqlx::Error> {
    let result = sqlx::query_file!("queries/data/match_info_main.sql", game_id)
        .fetch_optional(db.as_ref())
        .await?;

    if let Some(record) = result {
        let (team_1, team_2) = get_team_name(record.team_1, record.team_2, db).await?;
        Ok(Some(MatchMainInfo {
            place: record.location_desc,
            game_id: record.game_id,
            team_1,
            score_1: record.score_1,
            team_2,
            score_2: record.score_2,
            game_date: record.game_date,
        }))
    } else {
        Ok(None)
    }
}

#[derive(FromRow)]
struct RunPlayerInfo {
    run_position: i32,
    result: String,
    name: String,
    sname: String,
    player_id: i32
}

async fn select_game_runs_info(
    game_id: i32,
    db: &Arc<PgPool>,
) -> Result<Vec<RunPlayerInfo>, sqlx::Error> {
    sqlx::query_file_as!(RunPlayerInfo, "queries/data/match_info_games.sql", game_id)
        .fetch_all(db.as_ref())
        .await
}

#[derive(FromRow)]
struct RunTimesInfo {
    run_position: i32,
    time_integer: Option<i32>,
    time_decimal: Option<i32>,
}

async fn select_game_times(
    game_id: i32,
    db: &Arc<PgPool>,
) -> Result<Vec<RunTimesInfo>, sqlx::Error> {
    sqlx::query_file_as!(
        RunTimesInfo,
        "queries/data/match_info_run_times.sql",
        game_id
    )
    .fetch_all(db.as_ref())
    .await
}

fn combine_info(
    main_data: MatchMainInfo,
    game_runs: Vec<RunPlayerInfo>,
    run_times: Vec<RunTimesInfo>,
    player_scores: Vec<PlayerScoreInfo>,
) -> Result<MatchResult, &'static str> {
    let runs_by_time = run_times
        .iter()
        .max_by_key(|record| record.run_position)
        .ok_or_else(|| "Unable to find maximum runs_by_time.")?;
    let runs_by_game = game_runs
        .iter()
        .max_by_key(|record| record.run_position)
        .ok_or_else(|| "Unable to find maximum runs_by_game.")?;

    if runs_by_time.run_position != runs_by_game.run_position {
        return Err("Total number of runs does not match.");
    }

    let mut run_infos: HashMap<i32, Vec<(i32, String, String)>> = HashMap::new();

    for run in game_runs.iter() {
        let entry = match run_infos.get_mut(&run.run_position) {
            Some(entry) => entry,
            None => {
                let vector = Vec::new();
                run_infos.insert(run.run_position, vector);

                run_infos.get_mut(&run.run_position).unwrap()
            }
        };

        entry.push((run.player_id, format!("{} {}", run.name, run.sname), run.result.clone()));
    }

    let mut runs = Vec::new();

    for time in run_times {
        if let Some(time_decimal) = time.time_decimal {
            let time_integer = time
                .time_integer
                .ok_or_else(|| "Time nullity does not match.")?;

            let scores = run_infos
                .remove(&time.run_position)
                .ok_or_else(|| "Cannot find entry corresponding to given time.")?;

            // TODO: Unsafe casting -> should be checked if it can be done before.
            runs.push(RunInfo::new(
                time.run_position as u8,
                Some((time_integer as u32, time_decimal as u16)),
                scores,
            ));
        }
    }

    let mut player_infos: HashMap<(String, String), Vec<(u8, PlayerResult)>> = HashMap::new();

    for player_result in player_scores {
        let key = (player_result.name, player_result.sname);

        let entry = match player_infos.get_mut(&key) {
            Some(entry) => entry,
            None => {
                let vector = Vec::new();
                player_infos.insert(key.clone(), vector);

                player_infos.get_mut(&key).unwrap()
            }
        };

        entry.push((player_result.run as u8, player_result.score));
    }

    let mut players = Vec::new();

    for ((name, sname), scores) in player_infos.into_iter() {
        players.push(Player::new(name, sname, scores));
    }

    Ok(MatchResult::new(
        main_data.team_1,
        main_data.team_2,
        main_data.score_1 as u32,
        main_data.score_2 as u32,
        main_data.place,
        main_data.game_date,
        runs,
        players,
    ))
}

struct PlayerScoreInfo {
    name: String,
    sname: String,
    run: i32,
    score: PlayerResult,
}

async fn select_player_scores(
    game_id: i32,
    db: &Arc<PgPool>,
) -> Result<Option<Vec<PlayerScoreInfo>>, sqlx::Error> {
    let query_result = sqlx::query_file!("queries/data/match_info_player_score.sql", game_id)
        .fetch_all(db.as_ref())
        .await?;

    let mut result_vec = Vec::new();

    for record in query_result {
        match PlayerResult::from_str(&record.score) {
            Some(result) => {
                result_vec.push(PlayerScoreInfo {
                    name: record.name,
                    sname: record.sname,
                    run: record.round,
                    score: result,
                });
            }
            None => {
                return Ok(None);
            }
        }
    }

    Ok(Some(result_vec))
}

pub async fn match_info_handler(
    State(db): State<Arc<PgPool>>,
    Json(info): Json<HandlerInfo>,
) -> impl IntoResponse {
    let match_main_data = match select_main_data(info.match_id, &db).await {
        Ok(None) => return (StatusCode::NOT_FOUND).into_response(),
        Ok(Some(record)) => record,
        Err(e) => {
            tracing::error!(
                "Error retured from database while querying main match info. Error = [{e:?}]"
            );

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    let game_runs = match select_game_runs_info(info.match_id, &db).await {
        Ok(vec) => vec,
        Err(e) => {
            tracing::error!(
                "Error returned from database while querying game runs info. Error = [{e:?}]"
            );

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    let run_times = match select_game_times(info.match_id, &db).await {
        Ok(vec) => vec,
        Err(e) => {
            tracing::error!(
                "Error returned from database while querying runs time. Error = [{e:?}]"
            );

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    let player_scores = match select_player_scores(info.match_id, &db).await {
        Ok(Some(vec)) => vec,
        Ok(None) => {
            tracing::error!("None value returned while querying about player scores.");

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
        Err(e) => {
            tracing::error!(
                "Error returned from database while querying about player scores. Error = [{e:?}]"
            );

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    match combine_info(match_main_data, game_runs, run_times, player_scores) {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => {
            tracing::error!("Error while combining results. Error = [{e:?}]");

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

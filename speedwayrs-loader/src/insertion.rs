use std::{collections::HashMap, sync::Arc};

use crate::scraper_types::{GameInfo, Player, Team};
use speedwayrs_types::PlayerResult;
use sqlx::{Executor, PgExecutor, PgPool};

async fn check_player(name: &str, sname: &str, db: &PgPool) -> Result<i32, sqlx::Error> {
    let possible_id = sqlx::query_file!("queries/player_check.sql", name, sname)
        .fetch_optional(db)
        .await?;

    match possible_id {
        Some(id) => Ok(id.player_id),
        None => {
            let insert_stmt = sqlx::query_file!("queries/player_insert.sql", name, sname)
                .fetch_one(db)
                .await;

            match insert_stmt {
                Ok(record) => Ok(record.player_id),
                Err(e) => {
                    if let Some(db_error) = e.as_database_error() {
                        if let Some(error_string) = db_error.code() {
                            if error_string.as_bytes() == "23505".as_bytes() {
                                let id = sqlx::query_file!("queries/player_check.sql", name, sname)
                                    .fetch_one(db)
                                    .await?;

                                return Ok(id.player_id);
                            }
                        }
                    }

                    Err(e)
                }
            }
        }
    }
}

async fn check_team(name: &str, db: &PgPool) -> Result<i32, sqlx::Error> {
    let name = name.trim();

    let possible_id = sqlx::query_file!("queries/team_check.sql", name)
        .fetch_optional(db)
        .await?;

    match possible_id {
        Some(team) => Ok(team.team_id),
        None => {
            let insert_stmt = sqlx::query_file!("queries/team_insert.sql", name)
                .fetch_one(db)
                .await;

            match insert_stmt {
                Ok(record) => Ok(record.team_id),
                Err(e) => {
                    if let Some(db_error) = e.as_database_error() {
                        if let Some(error_string) = db_error.code() {
                            if error_string.as_bytes() == "23505".as_bytes() {
                                let id = sqlx::query_file!("queries/team_check.sql", name)
                                    .fetch_one(db)
                                    .await;

                                if let Err(e) = &id {
                                    eprintln!("Error while checking team {name}. Error = [{e:?}]");
                                }

                                let id = id?;

                                return Ok(id.team_id);
                            }
                        }
                    }

                    Err(e)
                }
            }
        }
    }
}

async fn check_place(description: &str, db: &PgPool) -> Result<i32, sqlx::Error> {
    let possible_id = sqlx::query_file!("queries/place_check.sql", description)
        .fetch_optional(db)
        .await?;

    match possible_id {
        Some(place) => Ok(place.stadium_id),
        None => {
            let insert_stmt = sqlx::query_file!("queries/place_insert.sql", description)
                .fetch_one(db)
                .await;

            match insert_stmt {
                Ok(record) => Ok(record.stadium_id),
                Err(e) => {
                    if let Some(db_error) = e.as_database_error() {
                        if let Some(error_string) = db_error.code() {
                            if error_string.as_bytes() == "23505".as_bytes() {
                                let id = sqlx::query_file!("queries/place_check.sql", description)
                                    .fetch_one(db)
                                    .await;

                                if let Err(e) = &id {
                                    eprintln!("Error while checking stadium {description}. Error = [{e:?}]");
                                }

                                let id = id?;

                                return Ok(id.stadium_id);
                            }
                        }
                    }

                    Err(e)
                }
            }
        }
    }
}

async fn map_players<'a>(
    players: &'a [Player],
    db: &Arc<PgPool>,
) -> Result<HashMap<(&'a str, &'a str), i32>, sqlx::Error> {
    let mut map = HashMap::new();

    for player in players {
        let id = check_player(player.name(), player.surname(), db).await?;

        map.insert((player.name().into(), player.surname().into()), id);
    }

    Ok(map)
}

async fn map_to_ids<'a, 'b>(
    team1: &'a Team,
    team2: &'b Team,
    db: &Arc<PgPool>,
) -> Result<
    (
        HashMap<(&'a str, &'a str), i32>,
        HashMap<(&'b str, &'b str), i32>,
    ),
    sqlx::Error,
> {
    Ok((
        map_players(team1.players(), db).await?,
        map_players(team2.players(), db).await?,
    ))
}

async fn insert_game<'a, T: PgExecutor<'a>>(
    payload: &GameInfo,
    team1_id: i32,
    team2_id: i32,
    stadium: i32,
    db: T,
) -> Result<i32, sqlx::Error> {
    let date = payload.date().assume_utc();

    let insert_result = sqlx::query_file!(
        "queries/insert_game.sql",
        team1_id,
        payload.team_one().score() as i32,
        payload.team_two().score() as i32,
        team2_id,
        stadium,
        date
    )
    .fetch_one(db)
    .await?;

    Ok(insert_result.game_id)
}

async fn insert_player_score<'a, T: PgExecutor<'a>>(
    player: i32,
    game: i32,
    round: i32,
    score: &PlayerResult,
    db: T,
) -> Result<(), sqlx::Error> {
    let score_str = score.to_string();

    sqlx::query_file!(
        "queries/insert_player_score.sql",
        game,
        player,
        round,
        score_str
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn create_run<'a, T: PgExecutor<'a>>(
    position: i32,
    time_integer: Option<i32>,
    time_decimal: Option<i32>,
    game_id: i32,
    db: T,
) -> Result<i64, sqlx::Error> {
    let query_result = sqlx::query_file!(
        "queries/create_run.sql",
        position,
        time_integer,
        time_decimal,
        game_id
    )
    .fetch_one(db)
    .await?;

    Ok(query_result.id)
}

async fn insert_run_squad_score<'a, T: PgExecutor<'a>>(
    run: i64,
    player: i32,
    result: &PlayerResult,
    db: T,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "queries/insert_run_squad_score.sql",
        run,
        player,
        result.to_string()
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn insert_into_database(db: Arc<PgPool>, payload: GameInfo) -> Result<(), sqlx::Error> {
    let team_1_id = check_team(payload.team_one().name(), &db).await?;
    let team_2_id = check_team(payload.team_two().name(), &db).await?;

    let stadium = check_place(payload.place(), &db).await?;

    eprintln!("Initial checking done.");

    let (mut main_hash_map, id2) = map_to_ids(payload.team_one(), payload.team_two(), &db).await?;

    main_hash_map.extend(id2.into_iter());

    let mut transaction = db.begin().await?;

    let game_id = insert_game(&payload, team_1_id, team_2_id, stadium, &mut transaction).await?;

    for player in payload
        .team_one()
        .players()
        .iter()
        .chain(payload.team_two().players().iter())
        .filter(|entry| {
            !entry.name().eq_ignore_ascii_case("Brak")
                || !entry.surname().eq_ignore_ascii_case("zawodnika")
        })
    {
        let key = (player.name(), player.surname());
        let id = main_hash_map.get(&key).unwrap();

        for (index, score) in player.scores().iter().enumerate() {
            insert_player_score(*id, game_id, index as i32, score, &mut transaction).await?;
        }
    }

    for run in payload.runs() {
        let (time_int, time_dec) = run.time();
        let run_id = create_run(
            run.position() as i32,
            time_int,
            time_dec,
            game_id,
            &mut transaction,
        )
        .await?;

        for score in run.player_scores() {
            let player_id = main_hash_map.get(&score.name());

            if let Some(player_id) = player_id {
                insert_run_squad_score(
                    run_id,
                    *player_id,
                    score.score(),
                    &mut transaction,
                )
                .await?;
            }
        }
    }

    transaction.commit().await?;

    Ok(())
}

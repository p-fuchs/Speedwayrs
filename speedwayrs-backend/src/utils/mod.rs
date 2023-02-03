mod chat;

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::post, Extension, Json, Router};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::session::AuthStatus;

#[derive(Deserialize, Debug)]
pub struct LikeInfo {
    team_id: Option<i32>,
    player_id: Option<i32>,
}

#[derive(Serialize)]
pub struct LikeResponse {
    team_like: Option<bool>,
    player_like: Option<bool>,
}

#[axum_macros::debug_handler]
async fn like(
    State(db): State<Arc<PgPool>>,
    Extension(auth_info): Extension<Arc<AuthStatus>>,
    Json(form): Json<LikeInfo>,
) -> impl IntoResponse {
    let username;
    let mut like_response = LikeResponse {
        team_like: None,
        player_like: None,
    };

    match auth_info.as_ref() {
        AuthStatus::NonAuthenticated => {
            return (StatusCode::UNAUTHORIZED).into_response();
        }
        AuthStatus::Authenticated(user) => {
            username = user;
        }
    }

    tracing::info!("Like log = [{form:?}]");

    if let Some(team_id) = form.team_id {
        let check_result =
            sqlx::query_file!("queries/utils/check_like_team.sql", username, team_id)
                .fetch_optional(db.as_ref())
                .await;

        match check_result {
            Err(e) => {
                tracing::error!(
                    "Error returned from database while checking user team like. Error = [{e:?}]"
                );
                return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
            Ok(None) => {
                let query_result =
                    sqlx::query_file!("queries/utils/post_like_team.sql", username, team_id)
                        .execute(db.as_ref())
                        .await;

                if query_result.is_err() {
                    tracing::error!(
                "Error returned from database while liking user. Error = [{query_result:?}]"
            );
                    return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }

                like_response.team_like = Some(true);
            }
            Ok(Some(_)) => {
                let unlike_result =
                    sqlx::query_file!("queries/utils/remove_like_team.sql", username, team_id)
                        .execute(db.as_ref())
                        .await;
                if unlike_result.is_err() {
                    tracing::error!(
                "Error returned from database while liking user. Error = [{unlike_result:?}]"
            );
                    return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }

                like_response.team_like = Some(false);
            }
        }
    }

    if let Some(player_id) = form.player_id {
        let check_result =
            sqlx::query_file!("queries/utils/check_like_player.sql", username, player_id)
                .fetch_optional(db.as_ref())
                .await;

        tracing::debug!("Form player_id branch. Result = [{check_result:?}]");

        match check_result {
            Err(e) => {
                tracing::error!(
                    "Error returned from database while checking user team like. Error = [{e:?}]"
                );
                return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
            Ok(None) => {
                let query_result =
                    sqlx::query_file!("queries/utils/post_like_player.sql", username, player_id)
                        .execute(db.as_ref())
                        .await;

                if query_result.is_err() {
                    tracing::error!(
                "Error returned from database while liking user. Error = [{query_result:?}]"
            );
                    return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }

                like_response.player_like = Some(true);
            }
            Ok(Some(_)) => {
                let unlike_result =
                    sqlx::query_file!("queries/utils/remove_like_player.sql", username, player_id)
                        .execute(db.as_ref())
                        .await;
                if unlike_result.is_err() {
                    tracing::error!(
                "Error returned from database while liking user. Error = [{unlike_result:?}]"
            );
                    return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }

                like_response.player_like = Some(false);
            }
        }
    }

    (StatusCode::OK, Json(like_response)).into_response()
}

pub fn utils_router() -> Router<crate::AppData> {
    Router::new().route("/like", post(like))
        .nest("/chat", chat::chat_router())
    }

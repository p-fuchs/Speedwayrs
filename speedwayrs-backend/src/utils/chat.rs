use axum::{extract::State, Json, Router, routing::post, response::IntoResponse};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;
use std::{sync::Arc, time::SystemTime};

use crate::session::SESSION_COOKIE;

#[derive(Deserialize)]
struct MessageInfo {
    message: String
}

fn current_time() -> OffsetDateTime {
    let time = SystemTime::now();

    OffsetDateTime::from(time)
}

#[debug_handler]
async fn post_message(State(db): State<Arc<PgPool>>, jar: CookieJar, Json(msg): Json<MessageInfo>) -> impl IntoResponse {
    let uuid = jar.get(SESSION_COOKIE).expect("Session middleware not working properly.").value();
    let uuid = Uuid::parse_str(uuid).unwrap();

    let query_username = sqlx::query_file!("queries/session_select.sql", uuid)
        .fetch_optional(db.as_ref())
        .await;

    match query_username {
        Err(e) => {
            tracing::error!("Error returned from database while checking user. Error = [{e:?}]");

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
        Ok(None) => {
            (StatusCode::UNAUTHORIZED).into_response()
        }
        Ok(Some(record)) => {
            let current_date = current_time();

            let query_insert = sqlx::query_file!("queries/utils/post_message.sql", record.username, msg.message, current_date)
                .execute(db.as_ref())
                .await;

            if let Err(e) = query_insert {
                tracing::error!("Error returned from database while posting message. Error = [{e:?}]");

                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            } else {
                (StatusCode::OK).into_response()
            }
        }
    }
}

const MESSAGES_ON_PAGE: i64 = 10;

#[derive(Deserialize)]
struct GetMessageInfo {
    page: i64 
}

#[derive(Serialize, FromRow)]
struct Message {
    username: String,
    time: OffsetDateTime,
    message: String
}

async fn messages(State(db): State<Arc<PgPool>>, Json(msg_info): Json<GetMessageInfo>) -> impl IntoResponse {
    let query_message = sqlx::query_file_as!(Message, "queries/utils/get_messages.sql", (msg_info.page - 1) * MESSAGES_ON_PAGE, MESSAGES_ON_PAGE)
        .fetch_all(db.as_ref())
        .await;

    match query_message {
        Err(e) => {
            tracing::error!("Error returned from database while checking messages. Error = [{e:?}]");
            
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
        Ok(records) => {
            (StatusCode::OK, Json(records)).into_response()
        }
    }
}

pub fn chat_router() -> Router<crate::AppData> {
    Router::new()
        .route("/post_message", post(post_message))
        .route("/messages", post(messages))
}

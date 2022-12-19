use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::{HeaderValue, Request},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Router, routing::get,
};
use axum_extra::extract::{cookie::{Cookie, SameSite}, CookieJar};
use http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::AppData;

pub const SESSION_COOKIE: &'static str = "srs-session";
const SESSION_EXPIRATION: Duration = Duration::from_secs(60 * 60);

pub enum AuthStatus {
    NonAuthenticated,
    Authenticated(String),
}

impl From<Option<String>> for AuthStatus {
    fn from(username: Option<String>) -> Self {
        match username {
            None => Self::NonAuthenticated,
            Some(uname) => Self::Authenticated(uname),
        }
    }
}

pub async fn login_session(
    pool: &Arc<PgPool>,
    uuid: &Uuid,
    username: &str,
) -> Result<time::OffsetDateTime> {
    let expiration: sqlx::Result<_> = sqlx::query_file!("queries/session_expiration.sql", uuid)
        .fetch_one(pool.as_ref())
        .await;

    match expiration {
        Err(e) => {
            tracing::error!("Database session expiration reported error: {:?}", e);

            Err(e.into())
        }
        Ok(expiration_row) => {
            let future_expiration = expiration_row.expiration + SESSION_EXPIRATION;

            let update: sqlx::Result<_> = sqlx::query_file!(
                "queries/session_update.sql",
                uuid,
                username,
                future_expiration
            )
            .execute(pool.as_ref())
            .await;

            match update {
                Err(e) => {
                    tracing::error!("Database session update reported error: {:?}", e);

                    Err(e.into())
                }
                Ok(_) => Ok(future_expiration),
            }
        }
    }
}

async fn validate_session(
    pool: &Arc<PgPool>,
    uuid: Uuid,
) -> (Option<String>, time::OffsetDateTime) {
    loop {
        let query: sqlx::Result<Option<_>> = sqlx::query_file!("queries/session_select.sql", uuid)
            .fetch_optional(pool.as_ref())
            .await;

        let expiration = time::OffsetDateTime::now_utc() + SESSION_EXPIRATION;

        match query {
            Err(e) => {
                tracing::trace!("Non final error from database: {:?}", e);
                tokio::time::sleep(Duration::from_millis(5000)).await;
            }
            Ok(possible_row) => match possible_row {
                None => {
                    let query: sqlx::Result<_> =
                        sqlx::query_file!("queries/session_insert.sql", uuid, expiration)
                            .execute(pool.as_ref())
                            .await;

                    if let Err(e) = query {
                        tracing::trace!("Non final error from database: {:?}", e);
                        tokio::time::sleep(Duration::from_millis(5000)).await;
                    }
                }
                Some(row) => {
                    let old_expiration: time::OffsetDateTime = row.expiration;
                    let current_time = time::OffsetDateTime::now_utc();
                    let future_username;

                    if current_time > old_expiration {
                        future_username = None;
                    } else {
                        future_username = row.username;
                    }

                    let query: sqlx::Result<_> = sqlx::query_file!(
                        "queries/session_update.sql",
                        uuid,
                        future_username,
                        current_time + SESSION_EXPIRATION
                    )
                    .execute(pool.as_ref())
                    .await;

                    if let Err(e) = query {
                        tracing::trace!("Non final error from database: {:?}", e);
                        tokio::time::sleep(Duration::from_millis(5000)).await;
                    } else {
                        return (future_username, current_time + SESSION_EXPIRATION);
                    }
                }
            },
        }
    }
}

async fn create_new_session(pool: &Arc<PgPool>) -> (Uuid, time::OffsetDateTime) {
    loop {
        let new_uuid = Uuid::new_v4();
        let expiration = time::OffsetDateTime::now_utc() + SESSION_EXPIRATION;

        let query: sqlx::Result<_> =
            sqlx::query_file!("queries/session_insert.sql", new_uuid, expiration)
                .execute(pool.as_ref())
                .await;

        match query {
            Ok(_) => {
                return (new_uuid, expiration);
            }
            Err(e) => match e {
                sqlx::Error::Database(db_error) => {
                    tracing::trace!("Error from database: {:?}", db_error);
                }
                other => {
                    tracing::trace!("Non final error from database: {:?}", other);
                    tokio::time::sleep(Duration::from_millis(5000)).await;
                }
            },
        }
    }
}

pub async fn session_management<B>(
    State(pg_pool): State<Arc<PgPool>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Response {
    let mut cookie_jar = axum_extra::extract::CookieJar::from_headers(req.headers());

    if let Some(cookie) = cookie_jar.get(SESSION_COOKIE) {
        let session_id = cookie.value();

        match Uuid::parse_str(session_id) {
            Err(_) => {
                // We act as we don't have session id
                // FIXME: We should check if cookie value is appropriate.
                req.headers_mut().remove(http::header::COOKIE);
                cookie_jar = cookie_jar.remove(Cookie::named(SESSION_COOKIE));
            }
            Ok(uuid) => {
                let (_, expiration_time) = validate_session(&pg_pool, uuid).await;

                cookie_jar = cookie_jar.remove(Cookie::named(SESSION_COOKIE));
                let cookie = Cookie::build(SESSION_COOKIE, uuid.to_string())
                    .expires(expiration_time)
                    .permanent()
                    .path("/")
                    .same_site(SameSite::None)
                    .http_only(true)
                    .finish();

                cookie_jar = cookie_jar.add(cookie);

                let next_r = next.run(req).await;
                return (cookie_jar, next_r).into_response();
            }
        }
    }
    // At this point we are unauthenticated user (with no session cookie)
    let (uuid, expiration_time) = create_new_session(&pg_pool).await;
    let cookie = Cookie::build(SESSION_COOKIE, uuid.to_string())
        .expires(expiration_time)
        .permanent()
        .path("/")
        .same_site(SameSite::None)
        .secure(false)
        .http_only(true)
        .finish();

    cookie_jar = cookie_jar.add(cookie);

    req.headers_mut().insert(
        http::header::COOKIE,
        HeaderValue::from_str(&format!("{}={}", SESSION_COOKIE, uuid)).unwrap(),
    );

    let auth_extension = Extension(AuthStatus::NonAuthenticated);
    req.extensions_mut().insert(auth_extension);

    let next_r = next.run(req).await;

    (cookie_jar, next_r).into_response()
}

// Returning server error
async fn session_info(State(pg_pool): State<Arc<PgPool>>, jar: CookieJar) -> impl IntoResponse {
    // It must not panic - ensured by middleware.
    let uuid = Uuid::parse_str(jar.get(SESSION_COOKIE).unwrap().value()).unwrap();

    let query: sqlx::Result<_> = sqlx::query_file!("queries/session_select.sql", uuid)
        .fetch_one(pg_pool.as_ref())
        .await;

    match query {
        Ok(username) => {
            (StatusCode::OK, username.username.unwrap_or_else(|| "".to_owned())).into_response()
        }
        Err(e) => {
            tracing::error!("Error while requesting session_info from database: {e:?}");

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

pub fn session_router() -> Router<AppData> {
    Router::new()
        .route("/", get(session_info))
}
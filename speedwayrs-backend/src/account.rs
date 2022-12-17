use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher};
use axum::{
    extract::Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::AppData;

const USERNAME_MAX_LEN: usize = 50;

fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut rand::rngs::OsRng);

    let argon = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::default(),
    );
    let hash = argon.hash_password(password.as_bytes(), &salt).unwrap();

    hash.to_string()
}

#[derive(Deserialize, Clone)]
struct SignupForm {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
enum RegisterError {
    FieldTaken {
        username_taken: bool,
        email_taken: bool,
    },
    ServerError,
    FieldInvalid {
        username_invalid: bool,
        email_invalid: bool,
    },
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> axum::response::Response {
        match &self {
            Self::FieldTaken { .. } => (StatusCode::CONFLICT, Json(self)).into_response(),
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::FieldInvalid { .. } => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response()
            }
        }
    }
}

async fn register_check(pg_pool: &Arc<PgPool>, form: &SignupForm) -> Result<(), RegisterError> {
    let username_invalid = form.username.len() > USERNAME_MAX_LEN;
    let email_invalid = !email_address::EmailAddress::is_valid(&form.email);

    if username_invalid || email_invalid {
        return Err(RegisterError::FieldInvalid {
            username_invalid,
            email_invalid,
        });
    }

    let check_query: Result<Vec<_>, sqlx::Error> =
        sqlx::query_file!("queries/register_check.sql", form.username, form.email)
            .fetch_all(pg_pool.as_ref())
            .await;

    match check_query {
        Ok(rows) => {
            let mut username_taken = false;
            let mut email_taken = false;

            for row in rows {
                username_taken |= row.username == form.username;
                email_taken |= row.email == form.email;
            }

            if username_taken || email_taken {
                Err(RegisterError::FieldTaken {
                    username_taken,
                    email_taken,
                })
            } else {
                Ok(())
            }
        }
        Err(e) => {
            tracing::error!("Database unsuspected error: {e:?}.");
            Err(RegisterError::ServerError)
        }
    }
}

#[axum_macros::debug_handler]
async fn register(
    State(pg_pool): State<Arc<PgPool>>,
    Json(form): Json<SignupForm>,
) -> Result<StatusCode, RegisterError> {
    register_check(&pg_pool, &form).await?;

    let hash = hash_password(form.password);

    let query_result: Result<_, sqlx::Error> =
        sqlx::query_file!("queries/register_user.sql", form.username, hash, form.email)
            .execute(pg_pool.as_ref())
            .await;

    if let Err(e) = query_result {
        tracing::error!("Database unsuspected error: {e:?}.");
        Err(RegisterError::ServerError)
    } else {
        Ok(StatusCode::CREATED)
    }
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Serialize)]
enum LoginError {
    InvalidField,
    ServerError,
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InvalidField => (StatusCode::FORBIDDEN).into_response(),
            Self::ServerError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}

async fn login(
    State(pg_pool): State<Arc<PgPool>>,
    Json(form): Json<LoginForm>,
) -> Result<StatusCode, LoginError> {
    if form.username.len() > USERNAME_MAX_LEN {
        return Err(LoginError::InvalidField);
    }

    let query_result: Result<Option<_>, sqlx::Error> =
        sqlx::query_file!("queries/login_user.sql", form.username)
            .fetch_optional(pg_pool.as_ref())
            .await;

    match query_result {
        Err(e) => {
            tracing::error!("Database unsuspected error: {e:?}.");

            Err(LoginError::ServerError)
        }
        Ok(possible_row) => match possible_row {
            Some(row) => {
                let hash = PasswordHash::new(&row.password_hash).unwrap();
                let hasher_params = argon2::Params::try_from(&hash).unwrap();
                let hasher = Argon2::new(
                    argon2::Algorithm::Argon2id,
                    argon2::Version::V0x13,
                    hasher_params,
                );

                if hash.verify_password(&[&hasher], &form.password).is_err() {
                    Err(LoginError::InvalidField)
                } else {
                    Ok(StatusCode::OK)
                }
            }
            None => Err(LoginError::InvalidField),
        },
    }
}

pub fn users_router() -> Router<AppData> {
    Router::new()
        .route("/", post(register))
        .route("/", get(login))
}
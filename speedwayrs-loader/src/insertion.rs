use std::sync::Arc;

use speedwayrs_types::scraper_types::GameInfo;
use sqlx::PgPool;

pub async fn insert_into_database(db: Arc<PgPool>, payload: GameInfo) {}

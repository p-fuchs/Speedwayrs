mod insertion;
mod scraper_types;

use scraper_types::GameInfo;
use sqlx::PgPool;
use std::sync::Arc;
use std::{fmt::Debug, fs::File};
use tokio::sync::mpsc::{self, UnboundedReceiver};

enum LoaderTask {
    Load(GameInfo),
    End,
}

impl Debug for LoaderTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Loader")
    }
}

async fn loader(database: Arc<PgPool>, mut rx: UnboundedReceiver<LoaderTask>) {
    let mut task_set = tokio::task::JoinSet::new();

    while let Some(msg) = rx.recv().await {
        match msg {
            LoaderTask::Load(payload) => {
                let database = database.clone();

                task_set.spawn(async move {
                    let result = insertion::insert_into_database(database, payload).await;

                    if let Err(e) = result {
                        eprintln!("Error while inserting. Error = [{e:?}]");
                    }
                });
                // tokio::time::sleep(std::time::Duration::from_secs_f64(0.5)).await;
            }
            LoaderTask::End => {
                break;
            }
        }
    }

    while let Some(outcome) = task_set.join_next().await {
        if let Err(e) = outcome {
            eprintln!("Join error on tokio. Error = [{e:?}]");
        }
    }
}

fn main() -> Result<(), String> {
    dotenvy::dotenv().unwrap();

    // Iterator skipping program executable path.
    let mut args = std::env::args().skip(1);
    let database_str =
        std::env::var("LOADER_POSTGRES").expect("Unable to load LOADER_POSTGRES env variable.");

    let path = match args.next() {
        None => {
            println!("ERROR: First argument should be path to scraper's data!");

            return Err("Path to scraper's file not found.".into());
        }
        Some(path) => path,
    };

    let (tx, rx) = mpsc::unbounded_channel();

    let tokio_handle = std::thread::spawn(move || {
        let loader_runtime = tokio::runtime::Builder::new_current_thread()
            .worker_threads(3)
            .enable_all()
            .build()
            .expect("Unable to build tokio runtime.");

        loader_runtime.block_on(async move {
            let postgres_pool = match sqlx::postgres::PgPool::connect(&database_str).await {
                Err(e) => {
                    eprintln!("Error returned from database. Error = {e:?}");

                    std::process::exit(1);
                }
                Ok(pool) => pool,
            };

            let sync_pool = Arc::new(postgres_pool);

            loader(sync_pool, rx).await;
        });
    });

    let file =
        File::open(path).map_err(|e| format!("Unable to open scraper's file. Error = [{e:?}]"))?;

    let deserializer = serde_json::Deserializer::from_reader(&file).into_iter::<GameInfo>();

    let mut read = 0;
    for input in deserializer {
        match input {
            Err(e) => {
                if e.is_eof() {
                    break;
                } else {
                    eprintln!("Error while deserializing. Error = [{e:?}]");

                    return Err("Deserializing error.".into());
                }
            }
            Ok(input) => {
                read += 1;

                if let Err(e) = tx.send(LoaderTask::Load(input)) {
                    panic!("Error while sending task. Error = [{e:?}]");
                }
            }
        }
    }

    if let Err(e) = tx.send(LoaderTask::End) {
        panic!(
            "Error while sending end signal. Error = [{}]",
            e.to_string()
        );
    }

    eprintln!("Deserializer read {read} structs.");

    tokio_handle.join().map_err(|e| format!("{e:?}"))
}

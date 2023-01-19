mod insertion;

use speedwayrs_types::scraper_types::GameInfo;
use sqlx::PgPool;
use std::fs::File;
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver};

enum LoaderTask {
    Load(GameInfo),
    End,
}

async fn loader(database: Arc<PgPool>, mut rx: UnboundedReceiver<LoaderTask>) {
    let mut task_set = tokio::task::JoinSet::new();

    while let Some(msg) = rx.recv().await {
        match msg {
            LoaderTask::Load(payload) => {
                let database = database.clone();

                task_set.spawn(async move {
                    insertion::insert_into_database(database, payload);
                });
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
    // Iterator skipping program executable path.
    let mut args = std::env::args().skip(1);
    let database_str = std::env::var("LOADER_POSTGRES")
        .expect("Unable to load LOADER_POSTGRES env variable.");

    let path = match args.next() {
        None => {
            println!("ERROR: First argument should be path to scraper's data!");

            return Err("Path to scraper's file not found.".into());
        }
        Some(path) => path,
    };

    let (tx, rx) = mpsc::unbounded_channel();

    let tokio_handle = std::thread::spawn(move || {
        let loader_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(3)
            .build()
            .expect("Unable to build tokio runtime.");

        loader_runtime.block_on(async move {
            let postgres_pool = match sqlx::PgPool::connect(&database_str).await {
                Err(e) => {
                    eprintln!("Error connecting to database. Error = [{e:?}]");

                    std::process::exit(1);
                }
                Ok(pool) => pool
            };

            let sync_pool = Arc::new(postgres_pool);

            loader(sync_pool, rx).await;
        });
    });

    let file =
        File::open(path).map_err(|e| format!("Unable to open scraper's file. Error = [{e:?}]"))?;

    let deserializer = serde_json::Deserializer::from_reader(&file).into_iter::<GameInfo>();

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
                tx.send(LoaderTask::Load(input)).expect("Unable to send message to tokio runtime.");
            }
        }
    }

    tokio_handle.join().map_err(|e| format!("{e:?}"))
}

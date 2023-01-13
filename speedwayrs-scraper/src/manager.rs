use crate::{
    game::{GameSite, ScraperGameInfo},
    season::Season,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{mpsc::Receiver, RwLock, Arc},
};

use anyhow::{Context, Result};
use indicatif::{ProgressStyle, ProgressState, ProgressBar};
use scraper::Html;
use threadpool::ThreadPool;

use crate::config::ProgramConfig;
use crate::http::HttpRequester;

pub struct Manager {
    pool: ThreadPool,
    output_folder: PathBuf,
}

const FILE_NAME: &'static str = "scraping_result.json";

impl Manager {
    pub fn new(config: &ProgramConfig) -> Self {
        Self {
            pool: ThreadPool::new(config.concurrency()),
            output_folder: config.output_folder(),
        }
    }

    fn read_game_sites(&self) -> Result<Vec<GameSite>> {
        // Returns a vector of seasons along with their site links.
        let seasons = Season::parse_site()?;
        let mut games = Vec::new();

        for season in seasons {
            let season_source = HttpRequester::make_request(season.site())?;
            let season_html = Html::parse_document(&season_source);

            let mut game_info = GameSite::parse_match_schedule(&season_html)?;

            games.append(&mut game_info);
        }

        Ok(games)
    }

    pub fn begin_scraping(&self) -> Result<()> {
        let count = Arc::new(RwLock::new(0));
        let end = Arc::new(RwLock::new(false));

        let mut file_path = self.output_folder.clone();
        file_path.push(FILE_NAME);

        let file_link = std::fs::File::create(file_path)
            .context("Unable to create file with parsing results.")?;

        let (tx, rx) = std::sync::mpsc::channel();

        let end_rw = end.clone();
        let count_arc = count.clone();
        let saving_job = move |pb: ProgressBar, file: File, receiver: Receiver<Result<ScraperGameInfo>>| {
            let mut file_buffer = BufWriter::new(file);

            while let Ok(info) = receiver.recv() {
                if let Ok(info_ok) = info {
                    let serialized_info = serde_json::to_string_pretty(&info_ok).unwrap();

                    if let Err(e) = file_buffer.write_all(serialized_info.as_bytes()) {
                        eprintln!("ERROR: While reading to file = [{:?}]", e);
                    }

                    {
                        let mut count_write = count_arc.write().unwrap();
                        *count_write += 1;
                        pb.set_position(*count_write);
                    }
                } else {
                    eprintln!("Wczytano error: {:?}\n", info);
                }
            }

            let _ = file_buffer.flush();
            *end_rw.write().unwrap() = true;
        };

        let games = self.read_game_sites()?;

        let pb = ProgressBar::new(games.len().try_into().unwrap());
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
            
        std::thread::spawn(move || saving_job(pb, file_link, rx));


        for game in games {
            let tx_clone = tx.clone();

            self.pool.execute(move || {
                let game_source = HttpRequester::make_request(game.url());

                match game_source {
                    Err(e) => {
                        tx_clone.send(Err(e)).unwrap();
                    }
                    Ok(source) => {
                        let game_info = ScraperGameInfo::parse_site(&source, game.url());
                        tx_clone.send(game_info).unwrap();
                    }
                }
            });
        }

        self.pool.join();

        Ok(())
    }
}

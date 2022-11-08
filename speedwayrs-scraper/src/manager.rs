use crate::{
    game::{self, GameSite, ScraperGameInfo},
    season::Season,
};
use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use scraper::Html;
use threadpool::ThreadPool;

use crate::config::ProgramConfig;
use crate::http::HttpRequester;

pub struct Manager {
    pool: ThreadPool,
    output_file: PathBuf,
}

impl Manager {
    pub fn new(config: &ProgramConfig) -> Self {
        Self {
            pool: ThreadPool::new(config.concurrency()),
            output_file: config.output_file(),
        }
    }

    pub fn begin_scrapping(&mut self) -> Result<()> {
        // Returns a vector of seasons along with their site links.
        let seasons = Season::parse_site()?;
        let mut games = Vec::new();

        for season in seasons {
            let season_source = HttpRequester::make_request(season.site())?;
            let season_html = Html::parse_document(&season_source);

            let mut game_info = GameSite::parse_match_schedule(&season_html)?;

            games.append(&mut game_info);
        }

        let game_infos = Arc::new(Mutex::new(Vec::new()));

        for game in games {
            let game_infos_clone = game_infos.clone();

            self.pool.execute(move || {
                let game_source = HttpRequester::make_request(game.url());

                let mut game_infos_lock = game_infos_clone.lock().unwrap();
                match game_source {
                    Err(e) => {
                        game_infos_lock.push(Err(e));
                    }
                    Ok(source) => {
                        let game_info = ScraperGameInfo::parse_site(&source, game.url());
                        game_infos_lock.push(game_info);
                    }
                }
            });
        }

        // TODO: Usunąć
        {
            let gi_cl = game_infos.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let gi_lock = gi_cl.lock().unwrap();

                println!(
                    "ACTUAL GAMES PARSED: {}. LAST: {:?}",
                    gi_lock.len(),
                    gi_lock.iter().next()
                );
                let _ = std::io::stdout().flush();
            });
        }

        self.pool.join();

        let game_infos = game_infos.lock().unwrap();
        for g in game_infos.iter() {
            println!("PARSED {:?}", g);
        }

        Ok(())
    }
}

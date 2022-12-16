mod config;
mod game;
mod http;
mod manager;
mod season;
mod file;

const MAIN_SITE: &str = "https://sportowefakty.wp.pl/zuzel/pge-ekstraliga/terminarz";
const RELATIVE_MAIN: &str = "/zuzel/pge-ekstraliga/terminarz";
const BASE_SITE: &str = "https://sportowefakty.wp.pl";

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use manager::Manager;
use scraper::{Html, Selector};

use crate::config::ProgramConfig;

const CONNECT_SLEEPING_DURATION: Duration = Duration::from_secs(3);
const REQUEST_PAUSE_DURATION: Duration = Duration::from_millis(50);

fn main() -> Result<()> {
    let config = ProgramConfig::parse();
    http::HttpRequester::set_tick_interval(config.tick_interval())?;

    file::check_folder(config.output_folder())?;

    let manager = manager::Manager::new(&config);
    manager.begin_scraping()?;

    Ok(())
}

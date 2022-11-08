use std::{path::PathBuf, time::Duration};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Polish speedway match results scraper.")]
pub struct ProgramConfig {
    /// Sets number of threads to send HTTP requests.
    #[arg(default_value_t = 2, long, value_name = "THREADS")]
    concurrency: usize,

    /// Location of output file
    #[arg(long, short = 'o', required = true, value_name = "OUT_FILE")]
    output_file: PathBuf,

    /// Minimal duration between sending HTTP requests (in milliseconds as integer number).
    #[arg(long, short = 'i', value_name = "INTERVAL", default_value_t = 100)]
    tick_interval: u64,
}

impl ProgramConfig {
    /// Returns number of threads which can be used to send HTTP requests.
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    /// Returns path to output file.
    pub fn output_file(&self) -> PathBuf {
        self.output_file.clone()
    }

    /// Returns required interval between http requests.
    pub fn tick_interval(&self) -> Duration {
        Duration::from_millis(self.tick_interval)
    }
}

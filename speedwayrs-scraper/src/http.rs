use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Context, Result};
use once_cell::sync::OnceCell;

static REQUESTER: OnceCell<Arc<Mutex<HttpRequester>>> = OnceCell::new();
pub const DEFAULT_TICK_INTERVAL: Duration = Duration::from_millis(1);
const CONNECT_SLEEPING_DURATION: Duration = Duration::from_secs(3);

/// Struct used as http request limiter.
/// Only thread which have this struct can make http requests.
/// Used to not over overwhelm server with pings.
pub struct HttpRequester {
    last_request: Option<SystemTime>,
    tick_interval: Duration,
}

impl HttpRequester {
    fn new(tick_interval: Duration) -> Self {
        Self {
            last_request: None,
            tick_interval,
        }
    }

    fn default_new() -> Self {
        Self::new(DEFAULT_TICK_INTERVAL)
    }

    pub fn set_tick_interval(tick_interval: Duration) -> Result<()> {
        let set_result = REQUESTER.set(Arc::new(Mutex::new(Self::new(tick_interval))));

        if set_result.is_err() {
            Err(anyhow!("HttpRequester was already initialized!"))
        } else {
            Ok(())
        }
    }

    fn mutex() -> Arc<Mutex<Self>> {
        REQUESTER
            .get_or_init(|| Arc::new(Mutex::new(HttpRequester::default_new())))
            .clone()
    }

    pub fn make_request(url: &str) -> Result<String> {
        {
            let self_mutex = Self::mutex();
            let mut self_lock = self_mutex.lock().unwrap();

            if let Some(last_time) = self_lock.last_request {
                let elapsed_time = last_time.elapsed()?;

                if elapsed_time < self_lock.tick_interval {
                    std::thread::sleep(self_lock.tick_interval - elapsed_time);
                }
            }

            self_lock.last_request = Some(SystemTime::now());
        }

        loop {
            match reqwest::blocking::get(url) {
                Ok(site) => {
                    let body = site
                        .text()
                        .with_context(|| format!("Unable to get body of site [{url}]."))?;

                    return Ok(body);
                }

                Err(e) => {
                    if e.is_connect() {
                        std::thread::sleep(CONNECT_SLEEPING_DURATION)
                    } else {
                        return Err(e)
                            .with_context(|| format!("Unable to connect with site [{url}]."));
                    }
                }
            }
        }
    }
}

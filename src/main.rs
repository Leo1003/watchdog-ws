#[macro_use]
extern crate custom_error;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use log::LevelFilter;
use self::config::Configure;
use std::process::exit;
use ws::{Handler, Handshake, Request};
use url::Url;
use std::sync::Mutex;
use self::error::AppResult;

mod config;
mod error;

lazy_static! {
    static ref RETRIED: Mutex<u32> = Mutex::new(0);
}

fn main() -> AppResult<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    if cfg!(debug_assertions) {
        log::set_max_level(LevelFilter::Debug);
    } else {
        log::set_max_level(LevelFilter::Info);
    }

    let cfg = Configure::load()?;
    if cfg.url().is_empty() || cfg.token().is_empty() {
        error!("Token is empty! Application abort!");
        exit(1);
    }

    loop {
        ws::connect(cfg.url(), |_s| {
            debug!("TCP connect");
            WsHandler {
                token: cfg.token(),
            }
        }).map_err(|err| {
            error!("Failed to connect to watchdog!\n {}", err);
            err
        }).unwrap_or(());
        let mut l = RETRIED.lock().unwrap();
        *l += 1;
        if *l >= 3 {
            error!("Reached retries limit!");
            panic!();
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct WsHandler<'a> {
    token: &'a str,
}

impl Handler for WsHandler<'_> {
    fn build_request(&mut self, url: &Url) -> ws::Result<Request> {
        let tokenstring = "Bearer ".to_owned() + &self.token;
        let mut req = Request::from_url(url)?;
        req.headers_mut().push(("Authorization".to_owned(), tokenstring.into()));
        Ok(req)
    }

    fn on_open(&mut self, _shake: Handshake) -> ws::Result<()> {
        info!("Connection established.");
        *RETRIED.lock().unwrap() = 0;
        Ok(())
    }
}

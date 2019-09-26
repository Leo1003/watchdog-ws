#[macro_use]
extern crate custom_error;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use log::LevelFilter;
use self::config::Configure;
use std::process::exit;
use ws::{Handler, Handshake, Request, Sender, util::Token};
use url::Url;
use std::sync::Mutex;
use self::error::AppResult;

mod config;
mod error;

lazy_static! {
    static ref RETRIED: Mutex<u32> = Mutex::new(0);
}
const PING_CLOCK_EVENT: Token = Token(1);

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
        ws::connect(cfg.url(), |s| {
            debug!("TCP connect");
            WsHandler {
                socket: s,
                keepalive: cfg.keepalive_ms(),
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

#[derive(Debug, Clone)]
struct WsHandler<'a> {
    socket: Sender,
    keepalive: u64,
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

        self.socket.timeout(self.keepalive, PING_CLOCK_EVENT)?;
        Ok(())
    }
    fn on_timeout(&mut self, event: Token) -> ws::Result<()> {
        if event == PING_CLOCK_EVENT {
            debug!("PING_CLOCK_EVENT triggered!");
            // send ping
            self.socket.ping(Vec::new())?;
            // reschedule the timeout
            self.socket.timeout(self.keepalive, PING_CLOCK_EVENT)?;
            Ok(())
        } else {
            Err(ws::Error::new(ws::ErrorKind::Internal, "Invalid timeout token!"))
        }
    }
}

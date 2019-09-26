use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::num::NonZeroU64;

const CONFIG_FILE: &'static str = "config.toml";
const KEEPALIVE_DEF: u64 = 120;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Configure {
    url: String,
    server_token: String,
    keepalive: Option<NonZeroU64>,
}

impl Configure {
    pub fn load() -> AppResult<Self> {
        let config_str = match fs::read_to_string(CONFIG_FILE) {
            Ok(string) => string,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    let def_cfg = Self::default();
                    def_cfg.save()?;
                    warn!("Config file not found. A default config file has been generated.");
                    log::logger().flush();
                }
                return Err(e.into());
            }
        };
        Ok(toml::from_str(&config_str)?)
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn token(&self) -> &str {
        &self.server_token
    }

    pub fn keepalive(&self) -> u64 {
        if let Some(sec) = self.keepalive {
            sec.get()
        } else {
            KEEPALIVE_DEF
        }
    }

    pub fn keepalive_ms(&self) -> u64 {
        self.keepalive() * 1000
    }

    pub fn save(&self) -> AppResult<()> {
        let config_str = toml::to_string_pretty(self)?;
        fs::write(CONFIG_FILE, config_str)?;
        Ok(())
    }
}

impl Default for Configure {
    fn default() -> Self {
        Self {
            url: String::new(),
            server_token: String::new(),
            keepalive: None,
        }
    }
}


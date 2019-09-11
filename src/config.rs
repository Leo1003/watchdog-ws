use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;

const CONFIG_FILE: &'static str = "config.toml";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Configure {
    url: String,
    server_token: String,
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

    pub fn save(&self) -> AppResult<()> {
        let config_str = toml::to_string_pretty(self)?;
        fs::write(CONFIG_FILE, config_str)?;
        Ok(())
    }
}

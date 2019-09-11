pub type AppResult<T> = Result<T, AppError>;

custom_error! {pub AppError
    //Request { source: reqwest::Error } = "HTTP Request Error",
    IO { source: std::io::Error } = "I/O Error",
    TomlSerialize { source: toml::ser::Error } = "TOML Serialize Error",
    TomlDeserialize { source: toml::de::Error } = "TOML Deserialize Error",
    //Json { source: serde_json::error::Error } = "JSON Serialize/Deserialize Error",
    //Binary { source: bincode::Error } = "Binary Serialize/Deserialize Error",
    Custom { message: String } = "{message}",
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Custom {
            message: format!("{}", err),
        }
    }
}

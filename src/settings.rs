use config::{Config, ConfigError, File};
use serde::Deserialize;

static CONFIG_FILE_DEFAULTS: &str = "config/settings.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: Database,
    pub server: Server,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: i32,
    pub database: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub bind: String,
}

impl Settings {
    /// Reads settings from the config file.
    fn init() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        config.merge(File::with_name(CONFIG_FILE_DEFAULTS))?;

        config.try_into()
    }

    /// Returns the config as a struct.
    pub fn get() -> Self {
        Settings::init().unwrap()
    }

    /// Returns the postgres connection url generated from the config.
    pub fn get_database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.user,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }
}

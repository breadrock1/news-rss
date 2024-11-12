use crate::cache::config::CacheConfig;
use crate::crawler::config::CrawlerConfig;
use crate::feeds::config::TopicsConfig;
use crate::logger::LoggerConfig;
use crate::publish::config::PublishConfig;
use crate::server::config::ServerConfig;
use crate::storage::config::StorageConfig;

use config::{Config, ConfigError, Environment, File};
use getset::Getters;
use serde::Deserialize;

const CONFIG_PREFIX: &str = "NEWS_RSS";
const SERVICE_RUN_MODE: &str = "NEWS_RSS_RUN_MODE";

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ServiceConfig {
    logger: LoggerConfig,
    server: ServerConfig,
    cache: CacheConfig,
    publish: PublishConfig,
    topics: TopicsConfig,
    crawler: CrawlerConfig,
    storage: StorageConfig,
}

impl ServiceConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var(SERVICE_RUN_MODE).unwrap_or("development".into());

        let run_mode_file_path = format!("./config/{}", run_mode);
        let current_config_file = File::with_name(&run_mode_file_path);

        let settings = Config::builder()
            .add_source(File::with_name("./config/development"))
            .add_source(current_config_file.required(false))
            .add_source(Environment::with_prefix(CONFIG_PREFIX))
            .build()?;

        settings.try_deserialize()
    }
}

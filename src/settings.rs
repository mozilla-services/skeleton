//! Application settings objects and initialization

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

static DEFAULT_PORT: u16 = 8000;

/*
static KILOBYTE: u32 = 1024;
static MEGABYTE: u32 = KILOBYTE * KILOBYTE;
static GIGABYTE: u32 = MEGABYTE * 1_000;
*/

static PREFIX: &str = "skeleton";

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub debug: bool,
    pub port: u16,
    pub host: String,
    pub human_logs: bool,
    pub statsd_label: String,
    pub statsd_host: Option<String>,
    pub statsd_port: u16,
    pub actix_keep_alive: Option<u64>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            debug: false,
            port: DEFAULT_PORT,
            host: "127.0.0.1".to_owned(),
            human_logs: false,
            statsd_label: PREFIX.to_owned(),
            statsd_host: None,
            statsd_port: 8125,
            actix_keep_alive: None,
        }
    }
}

impl Settings {
    /// Load the settings from the config file if supplied, then the environment.
    pub fn with_env_and_config_file(filename: &Option<String>) -> Result<Self, ConfigError> {
        let mut config = Config::builder();

        // Merge the config file if supplied
        if let Some(config_filename) = filename {
            config = config.add_source(File::with_name(config_filename));
        }

        // Merge the environment overrides
        // While the prefix is currently case insensitive, it's traditional that
        // environment vars be UPPERCASE, this ensures that will continue should
        // Environment ever change their policy about case insensitivity.
        // This will accept environment variables specified as
        // `SYNC_FOO__BAR_VALUE="gorp"` as `foo.bar_value = "gorp"`
        config =
            config.add_source(Environment::with_prefix(&PREFIX.to_uppercase()).separator("__"));

        let built = config.build()?;

        built
            .try_deserialize::<Self>()
            .map_err(|error| match error {
                ConfigError::Message(error_msg) => {
                    println!("Bad configuration: {:?}", &error_msg);
                    println!("Please set in config file or use environment variable.");
                    println!(
                        "For example to set `database_url` use env var `{}_DATABASE_URL`\n",
                        PREFIX.to_uppercase()
                    );
                    error!("Configuration error: Value undefined {:?}", &error_msg);
                    ConfigError::NotFound(error_msg)
                }
                _ => {
                    error!("Configuration error: Other: {:?}", &error);
                    error
                }
            })
    }

    /// A simple banner for display of certain settings at startup
    pub fn banner(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

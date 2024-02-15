use log::error;
use serde::{self, Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UvmConfig {
    pub proxy: Option<String>,
    pub data_dir: Option<String>,
    pub go: Option<GeneralLanguageConfig>,
    pub node: Option<GeneralLanguageConfig>,
    pub java: Option<JavaConfig>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GeneralLanguageConfig {
    pub proxy: Option<String>,
    pub mirror: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct JavaConfig {
    pub proxy: Option<String>,
    pub mirror: Option<String>,
    pub default_vendor: Option<String>,
}

static DEFAULT_CONFIG: UvmConfig = UvmConfig {
    proxy: None,
    data_dir: None,
    go: None,
    node: None,
    java: None,
};

type Result<T> = std::result::Result<T, ConfigError>;

impl UvmConfig {
    pub fn load_config(home_dir: &Path) -> Result<UvmConfig> {
        let config_path = home_dir.join("config");
        let mut config_file;
        if !config_path.exists() || !config_path.is_file() {
            Ok(DEFAULT_CONFIG.clone())
        } else {
            config_file = fs::File::open(config_path)?;
            let mut content = String::new();
            config_file.read_to_string(&mut content)?;
            let config: UvmConfig = toml::from_str(&content)?;
            Ok(config)
        }
    }

    pub fn save(&self, home_dir: &Path) -> Result<()> {
        let config_path = home_dir.join("config");
        let mut config_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(config_path)?;
        let content = toml::to_string(self)?;
        config_file.write_all(content.as_bytes())?;
        Ok(())
    }
}

pub fn load_config(home_dir: &Path) -> UvmConfig {
    match UvmConfig::load_config(home_dir) {
        Ok(config) => config,
        Err(err) => {
            error!("failed to load configuration: {}", err);
            DEFAULT_CONFIG.clone()
        }
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("file system error: {0}")]
    Fs(#[from] std::io::Error),
    #[error("config file error: {0}")]
    ConfigRead(#[from] toml::de::Error),
    #[error("config file error: {0}")]
    ConfigConvert(#[from] toml::ser::Error),
}

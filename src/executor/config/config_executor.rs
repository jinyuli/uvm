use super::uvm_config::{ConfigError, UvmConfig};
use log::info;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use thiserror::Error;

pub struct ConfigContext {
    pub home_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConfigKey {
    Proxy,
}

pub struct ConfigExecutor {}

pub type Result<T> = std::result::Result<T, ConfigExecutorError>;

impl ConfigExecutor {
    pub fn load(&self, context: &ConfigContext) -> Result<UvmConfig> {
        let config = UvmConfig::load_config(&context.home_dir)?;
        Ok(config)
    }

    pub fn get(
        &self,
        keys: HashSet<ConfigKey>,
        context: &ConfigContext,
    ) -> Result<HashMap<ConfigKey, String>> {
        let config = self.load(context)?;
        let mut result = HashMap::new();
        for key in keys {
            match key {
                ConfigKey::Proxy => {
                    if let Some(proxy) = &config.proxy {
                        result.insert(key, proxy.clone());
                    }
                }
            }
        }
        Ok(result)
    }

    pub fn set(&self, kvs: HashMap<ConfigKey, String>, context: &ConfigContext) -> Result<()> {
        let mut config = self.load(context)?;
        for (key, value) in kvs {
            match key {
                ConfigKey::Proxy => {
                    config.proxy = Some(value);
                }
            }
        }
        info!("save config: {:?}", &config);
        Ok(())
    }

    pub fn del(&self, keys: HashSet<ConfigKey>, context: &ConfigContext) -> Result<()> {
        let mut config = self.load(context)?;
        for key in keys {
            match key {
                ConfigKey::Proxy => {
                    config.proxy = None;
                }
            }
        }
        config.save(&context.home_dir)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ConfigExecutorError {
    #[error("{0}")]
    ConfigError(#[from] ConfigError),
}

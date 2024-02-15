use std::collections::{HashMap, HashSet};
use crate::tool::logger::error;
use super::uvm_renderer::{ConfigRenderer, UvmRenderer};
use super::terminal::TerminalRenderer;
use crate::executor::{ConfigContext, ConfigKey, ConfigExecutor};
use colored::Colorize;

/// ConfigTerminalRenderer executes `config` commands and renders outputs on terminal.
pub struct ConfigTerminalRenderer {
    executor: ConfigExecutor,
}

impl ConfigTerminalRenderer {
    pub fn new(executor: ConfigExecutor) -> Self {
        ConfigTerminalRenderer{ executor }
    }
}

impl ConfigRenderer for ConfigTerminalRenderer {
    fn get(&self, keys: HashSet<ConfigKey>, context: ConfigContext) {
        match self.executor.get(keys, &context) {
            Ok(kvs) => {
                for (k, v) in kvs {
                    match k {
                        ConfigKey::Proxy => {
                            self.print_line(format!("proxy: {}", v));
                        }
                    }
                }
            },
            Err(err) => {
                error!("failed to execute `get` command:{}", err);
                self.print_line(format!("failed to execute `get` command:\n\t{}", err.to_string().red()));
            },
        }
    }

    fn set(&self, kvs: HashMap<ConfigKey, String>, context: ConfigContext) {
        match self.executor.set(kvs, &context) {
            Ok(_) => {},
            Err(err) => {
                error!("failed to execute `set` command:{}", err);
                self.print_line(format!("failed to execute `set` command:\n\t{}", err.to_string().red()));
            },
        }
    }

    fn del(&self, keys: HashSet<ConfigKey>, context: ConfigContext) {
        match self.executor.del(keys, &context) {
            Ok(_) => {},
            Err(err) => {
                error!("failed to execute `del` command:{}", err);
                self.print_line(format!("failed to execute `del` command:\n\t{}", err.to_string().red()));
            },
        }
    }
}

impl TerminalRenderer for ConfigTerminalRenderer {}

impl UvmRenderer for ConfigTerminalRenderer {}

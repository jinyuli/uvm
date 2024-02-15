use crate::executor::{LanguageContext, ExecutorContext};
use crate::executor::{ConfigContext, ConfigKey};
use std::collections::{HashMap, HashSet};

/// Renderer is used to render `Executor` output.
pub trait UvmRenderer {
}

/// LanguageRenderer executes `LanguageExecutor` and renders output.
pub trait LanguageRenderer<'a, C: LanguageContext>: UvmRenderer {
    fn list(&self, local_only: bool, context: &'a ExecutorContext<'a, C>);
    fn install(&self, version: String, context: &'a ExecutorContext<'a, C>);
    fn uninstall(&self, version: String, context: &'a ExecutorContext<'a, C>);
    fn select(&self, version: String, context: &'a ExecutorContext<'a, C>);
    fn unuse(&self, context: &'a ExecutorContext<'a, C>);
    fn venv(&self, version: String, dir_name: String, context: &'a ExecutorContext<'a, C>);
}

/// ConfigRenderer executes `ConfigExecutor` and renders output.
pub trait ConfigRenderer: UvmRenderer {
    fn get(&self, keys: HashSet<ConfigKey>, context: ConfigContext);
    fn set(&self, kvs: HashMap<ConfigKey, String>, context: ConfigContext);
    fn del(&self, keys: HashSet<ConfigKey>, context: ConfigContext);
}

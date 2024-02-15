use crate::tool::http::{self, HttpError};
use serde::Deserialize;
use serde_json::{self, Error as JsonError, Value};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, HtmlError>;

pub async fn parse_node_official(
    url: &'static str,
    proxy: &Option<String>,
) -> Result<Vec<VersionItem>> {
    let content = http::download_html(&url.to_string(), proxy).await?;
    let data: Vec<VersionItem> = serde_json::from_str(content.as_str())?;
    Ok(data)
}

#[derive(Deserialize, Debug)]
pub struct VersionItem {
    pub version: String,
    pub files: Vec<String>,
    pub npm: Option<String>,
    lts: Value,
}

impl VersionItem {
    pub fn is_lts(&self) -> bool {
        match self.lts {
            Value::Bool(b) => b,
            Value::String(_) => true,
            _ => false,
        }
    }
}

#[derive(Error, Debug)]
pub enum HtmlError {
    #[error("{0}")]
    Http(#[from] HttpError),
    #[error("{0}")]
    Json(#[from] JsonError),
}

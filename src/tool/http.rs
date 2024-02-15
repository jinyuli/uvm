use reqwest;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, HttpError>;

pub async fn download_html(url: &String, proxy: &Option<String>) -> Result<String> {
    let client = get_client(proxy).await?; 
    let body = client.get(url).send().await?.text().await?;
    Ok(body)
}

pub async fn download_file(url: &String, to_file: &Path, proxy: &Option<String>) -> Result<()> {
    let client = get_client(proxy).await?; 
    let body = client.get(url).send().await?.bytes().await?;
    let mut file = File::create(to_file)?;
    file.write_all(&body)?;
    Ok(())
}

async fn get_client(proxy: &Option<String>) -> Result<reqwest::Client> {
    let client = match proxy {
        Some(p) => {
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::http(p)?)
                .proxy(reqwest::Proxy::https(p)?)
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0")
                .build()?
        },
        None => {
            reqwest::Client::builder().build()?
        }
    };
    Ok(client)
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("file system error: {0}")]
    FileSystem(#[from] std::io::Error),
}
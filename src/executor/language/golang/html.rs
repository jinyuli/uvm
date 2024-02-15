use std::collections::HashMap;

use crate::tool::http;
use scraper::{ElementRef, Html, Selector};

pub static KEY_TYPE: &str = "type";
pub static KEY_URL: &str = "url";
pub static KEY_VERSION: &str = "version";
pub static KEY_FILE_NAME: &str = "file_name";
pub static KEY_OS: &str = "os";
pub static KEY_KIND: &str = "kind";
pub static KEY_ARCH: &str = "arch";
pub static KEY_CHECKSUM: &str = "checksum";
pub static KEY_SIZE: &str = "size";

pub static TYPE_STABLE: &str = "stable";
pub static TYPE_UNSTABLE: &str = "unstable";
pub static TYPE_ARCHIVE: &str = "archive";

pub type Result<T> = std::result::Result<T, http::HttpError>;

pub async fn parse_golang_official(
    url: &'static str,
    proxy: &Option<String>,
) -> Result<HashMap<String, Vec<HashMap<&'static str, String>>>> {
    let content = http::download_html(&url.to_string(), proxy).await?;
    // println!("golang official {}", content);
    let document = Html::parse_document(&content);
    let stable_selector = Selector::parse("#stable ~ div[id^=\"go\"]").unwrap();
    let unstable_selector = Selector::parse("#unstable ~ div[id^=\"go\"]").unwrap();
    let archive_selector = Selector::parse("#archive div[id^=\"go\"]").unwrap();

    let stable_tables: Vec<_> = document.select(&stable_selector).collect();
    let unstable_tables: Vec<_> = document.select(&unstable_selector).collect();
    let archive_tables: Vec<_> = document.select(&archive_selector).collect();

    let mut stable_versions = parse_golang_official_table(&stable_tables, TYPE_STABLE);
    let unstable_versions = parse_golang_official_table(&unstable_tables, TYPE_UNSTABLE);
    let archive_versions = parse_golang_official_table(&archive_tables, TYPE_ARCHIVE);
    for k in unstable_versions.keys() {
        stable_versions.remove(k);
    }
    stable_versions.extend(unstable_versions);
    stable_versions.extend(archive_versions);
    Ok(stable_versions)
}

fn parse_golang_official_table(
    version_tables: &Vec<ElementRef<'_>>,
    version_type: &str,
) -> HashMap<String, Vec<HashMap<&'static str, String>>> {
    let table_selector = Selector::parse("table.downloadtable").unwrap();
    let tr_selector = Selector::parse("tbody > tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let tt_selector = Selector::parse("tt").unwrap();
    let mut versions: HashMap<String, Vec<HashMap<&str, String>>> = HashMap::new();
    for element in version_tables {
        let version = match element.attr("id") {
            Some(v) => v.to_string(),
            None => continue,
        };
        let tables = element.select(&table_selector);
        for table in tables {
            let mut items = Vec::new();
            let trs: Vec<_> = table.select(&tr_selector).collect();
            for tr in trs {
                let tds: Vec<_> = tr.select(&td_selector).collect();
                if tds.len() < 6 {
                    continue;
                }
                let name_elem = tds[0];
                let a: Vec<_> = name_elem.select(&a_selector).collect();
                if a.is_empty() {
                    continue;
                }
                let tts: Vec<_> = tds[5].select(&tt_selector).collect();
                let checksum = if tts.is_empty() {
                    tds[5].inner_html()
                } else {
                    tts[0].inner_html()
                };

                // let url = a[0].attr("href");
                if let Some(url) = a[0].attr("href") {
                    let mut item = HashMap::new();
                    item.insert(KEY_TYPE, version_type.to_string());
                    item.insert(KEY_VERSION, version.clone());
                    item.insert(KEY_URL, url.to_string());
                    item.insert(KEY_FILE_NAME, a[0].inner_html());
                    item.insert(KEY_KIND, tds[1].inner_html());
                    item.insert(KEY_OS, tds[2].inner_html());
                    item.insert(KEY_ARCH, tds[3].inner_html());
                    item.insert(KEY_SIZE, tds[4].inner_html());
                    item.insert(KEY_CHECKSUM, checksum);
                    items.push(item);
                }
            }
            if !items.is_empty() {
                versions.insert(version.clone(), items);
            }
        }
    }
    versions
}

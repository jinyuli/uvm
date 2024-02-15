use super::version::{JavaPackage, JavaVersion, PackageKind};
use crate::tool::http;
use crate::tool::logger::debug;
use scraper::{ElementRef, Html, Selector};
use std::{borrow::BorrowMut, collections::HashMap};

pub type Result<T> = std::result::Result<T, http::HttpError>;

const JAVA_HOME: &str = "https://jdk.java.net/";
const JAVA_ARCHIVE: &str = "https://jdk.java.net/archive/";

const DOWNLOAD_URL_PREFIX: &str = "https://download.java.net/java";

pub async fn parse_openjdk(proxy: &Option<String>) -> Result<Vec<JavaVersion>> {
    let mut result = Vec::new();
    let mut archive_versions = parse_archive(JAVA_ARCHIVE, proxy).await?;
    result.append(&mut archive_versions);
    let mut early_access_versions = parse_home(proxy).await?;
    result.append(&mut early_access_versions);
    Ok(result)
}

async fn parse_home(proxy: &Option<String>) -> Result<Vec<JavaVersion>> {
    let content = http::download_html(&JAVA_HOME.to_string(), proxy).await?;
    let document = Html::parse_document(&content);
    let h1_selector = Selector::parse("h1").unwrap();
    let elements: Vec<_> = document.select(&h1_selector).collect();
    if elements.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for element in elements {
        if element.inner_html().contains("Early access") {
            let a_selector = Selector::parse("a").unwrap();
            let a_elements: Vec<_> = element.select(&a_selector).collect();
            for a in a_elements {
                if a.inner_html().contains("JDK") {
                    let mut href;
                    match a.attr("href") {
                        Some(h) => href = h.to_string(),
                        None => continue,
                    }
                    if href.starts_with('/') {
                        href = format!("https://jdk.java.net{}", href);
                    }
                    let mut versions = parse_early_access(href.as_str(), proxy).await?;
                    result.append(versions.as_mut());
                }
            }
        }
    }
    Ok(result)
}

async fn parse_early_access(url: &str, proxy: &Option<String>) -> Result<Vec<JavaVersion>> {
    let content = http::download_html(&url.to_string(), proxy).await?;
    let document = Html::parse_document(&content);
    let a_selector = Selector::parse("blockquote > table[summary^=\"builds\"] tr a").unwrap();
    let elements: Vec<_> = document.select(&a_selector).collect();
    if elements.is_empty() {
        return Ok(Vec::new());
    }

    let file_name_re = regex::Regex::new(
        r"openjdk-(?<version>[0-9.]+-\w+\+\w+)_(?<os>\w+)-(?<arch>[^_.]+)(_[^.]+)?\.(?<ext>.*)",
    )
    .unwrap();
    let result = parse_table(elements, file_name_re).await?;
    Ok(result)
}

/// https://download.java.net/java/GA/jdk18.0.1.1/65ae32619e2f40f3a9af3af1851d6e19/2/GPL/openjdk-18.0.1.1_windows-x64_bin.zip
async fn parse_archive(url: &'static str, proxy: &Option<String>) -> Result<Vec<JavaVersion>> {
    let content = http::download_html(&url.to_string(), proxy).await?;
    let document = Html::parse_document(&content);
    let a_selector = Selector::parse("#downloads > table[summary^=\"Downloads\"] tr a").unwrap();
    let elements: Vec<_> = document.select(&a_selector).collect();
    if elements.is_empty() {
        return Ok(Vec::new());
    }

    let file_name_re = regex::Regex::new(
        r"openjdk-(?<version>[0-9.]+)_(?<os>\w+)-(?<arch>[^._]+)(_[^.]+)?\.(?<ext>.*)",
    )
    .unwrap();
    let result = parse_table(elements, file_name_re).await?;
    Ok(result)
}

async fn parse_table(
    elements: Vec<ElementRef<'_>>,
    file_name_re: regex::Regex,
) -> Result<Vec<JavaVersion>> {
    let mut packages = Vec::new();
    let mut checksum_urls = HashMap::new();
    for element in &elements {
        let href = match element.attr("href") {
            Some(h) => h,
            None => continue,
        };
        if href.starts_with(DOWNLOAD_URL_PREFIX) && href.ends_with(".sha256") {
            let segments: Vec<_> = href.split('/').collect();
            if segments.is_empty() {
                continue;
            }
            let file_name = segments[segments.len() - 1];
            checksum_urls.insert(file_name[0..(file_name.len() - 7)].to_string(), href);
        }
    }
    for element in elements {
        let href = match element.attr("href") {
            Some(h) => h,
            None => continue,
        };
        if href.starts_with(DOWNLOAD_URL_PREFIX) {
            let segments: Vec<_> = href.split('/').collect();
            if segments.is_empty() {
                continue;
            }
            let file_name = segments[segments.len() - 1];
            let checksum_url = checksum_urls.get(file_name);
            match file_name_re.captures(file_name) {
                Some(caps) => {
                    match parse_archive_file(href, &caps, checksum_url) {
                        Some(pkg) => packages.push(pkg),
                        None => {
                            // debug!("failed to parse file to version: {}", file_name);
                            continue;
                        }
                    }
                }
                None => {
                    debug!("openjdk file name regex does not match");
                    continue;
                }
            }
        }
    }

    let mut tmp_map = HashMap::new();
    packages.into_iter().fold(
        tmp_map.borrow_mut(),
        |map: &mut HashMap<String, Vec<JavaPackage>>, pkg: JavaPackage| {
            if map.contains_key(&pkg.version) {
                if let Some(vec) = map.get_mut(&pkg.version) {
                    vec.push(pkg);
                }
            } else {
                map.insert(pkg.version.clone(), vec![pkg]);
            }
            map
        },
    );

    let java_versions: Vec<JavaVersion> = tmp_map
        .into_values()
        .filter_map(JavaVersion::from_openjdk)
        .collect();

    Ok(java_versions)
}

fn parse_archive_file(
    href: &str,
    caps: &regex::Captures<'_>,
    checksum_url: Option<&&str>,
) -> Option<JavaPackage> {
    let version = match caps.name("version") {
        Some(v) => v.as_str(),
        None => return None,
    };
    let os = match caps.name("os") {
        Some(v) => v.as_str(),
        None => return None,
    };
    let arch = match caps.name("arch") {
        Some(v) => v.as_str(),
        None => return None,
    };
    let ext = match caps.name("ext") {
        Some(v) => v.as_str(),
        None => return None,
    };

    let kind = PackageKind::from_str(ext);
    if kind == PackageKind::None {
        return None;
    }

    let checksum = checksum_url.map(|v| v.to_string());

    Some(JavaPackage {
        version: version.to_string(),
        url: href.to_string(),
        os: os.to_string(),
        arch: arch.to_string(),
        checksum_url: checksum,
        checksum_md5: None,
        checksum_sha256: None,
        kind,
    })
}

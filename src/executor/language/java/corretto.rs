use super::{
    version::{JavaPackage, JavaVersion, PackageKind},
    super::language_executor::Result,
};
use log::debug;
use octocrab::{self, params};

pub async fn parse_corretto() -> Result<Vec<JavaVersion>> {
    let client = octocrab::instance();
    let page = client
        .orgs("corretto")
        .list_repos()
        .repo_type(params::repos::Type::Sources)
        .per_page(50)
        .send()
        .await?;
    // debug!("repos: {:?}", page);
    let name_re = regex::Regex::new(r"^corretto-(?<version>\d+)$").unwrap();
    let repositories: Vec<_> = page.items.iter().filter_map(|r| {
        match name_re.captures(&r.name) {
            Some(caps) => {
                caps.get(0).map(|v| v.as_str())
            },
            None => None,
        }
    })
    .collect();

    let mut result = Vec::new();
    for repository in repositories {
        let release_result = client.repos("corretto", repository).releases().list().per_page(100).send().await?;
        let mut versions: Vec<_> = release_result.items.iter().filter_map(|item|{
            match &item.body {
                Some(body) => parse_table(body.as_str()),
                None => None,
            }
        }).collect();
        result.append(&mut versions);
    }

    Ok(result)
}

fn parse_table(body: &str) -> Option<JavaVersion> {
    let download_regex = regex::Regex::new(r"\[.*\]\((?<url>.*)\)").unwrap();
    let checksum_regex = regex::Regex::new(r"`(?<md5>[^`]*)`(\s+/<br\s+/>\s+`(?<sha256>[^`])`)?").unwrap();
    let packages: Vec<_> = body.split("\r\n").filter_map(|line| {
        let segments: Vec<_> = line.split('|').filter_map(|seg| {
            let s = seg.trim_matches(|c: char| c.is_whitespace() || c == '-');
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }).collect();
        if segments.len() < 4 || segments[1] != "JDK" {
            None
        } else {
            let url = match download_regex.captures(segments[2]) {
                Some(caps) => {
                    match caps.name("url") {
                        Some(u) => u.as_str(),
                        None => return None,
                    }
                },
                None => {
                    debug!("cannot match url: {}", segments[2]);
                    return None},
            };
            let (checksum_md5, checksum_sha256) = match checksum_regex.captures(segments[3]) {
                Some(caps) => {
                    let md5 = caps.name("md5").map(|v| v.as_str().to_string());
                    let sha256 = caps.name("sha256").map(|v| v.as_str().to_string());
                    (md5, sha256)
                },
                None => {
                    debug!("cannot match checksum: {}", segments[3]);
                    return None},
            };

            parse_corretto_url(url, checksum_md5, checksum_sha256)
        }
    }).collect();
    if packages.is_empty() {
        None
    } else {
        JavaVersion::from_corretto(packages)
    }
}

fn parse_corretto_url(href: &str, checksum_md5: Option<String>, checksum_sha256: Option<String>) -> Option<JavaPackage> {
    let splits: Vec<_> = href.split('/').collect();
    if splits.len() < 2 {
        return None;
    }

    // let version_re = regex::Regex::new(r"\d+\.\d+\.\d+(\.(?<build>.*))?").unwrap();
    // let version = splits[splits.len() - 2];
    // let version_str = match version_re.captures(version) {
    //     Some(caps) => {
    //         match caps.name("build") {
    //             Some(build) => {
    //                 let build_str = build.as_str();
    //                 format!("{}+{}", &version[0..(version.len() - build_str.len() - 1)], build_str)
    //             },
    //             None => version.to_string(),
    //         }
    //     },
    //     None => version.to_string(),
    // };
    let version_str = splits[splits.len() - 2];
    let file_name = splits[splits.len() - 1];

    // debug!("\t file name: {}, version: {}", file_name, version_str);
    let file_name_re = regex::Regex::new(r"amazon-corretto-([\d.]+)-(?<os>[^-.]+)-(?<arch>[^-.]+)(-(jdk|jre))?\.(?<ext>.*)").unwrap();
    match file_name_re.captures(file_name) {
        Some(caps) => {
            parse_github_file(href, version_str, &caps, checksum_md5, checksum_sha256)
        },
        None => {
            // debug!("file name's pattern is not recoganized: {}", file_name);
            None
        },
    }
}

fn parse_github_file(
    href: &str,
    version: &str,
    caps: &regex::Captures<'_>,
    checksum_md5: Option<String>,
    checksum_sha256: Option<String>,
) -> Option<JavaPackage> {
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

    Some(JavaPackage {
        version: version.to_string(),
        url: href.to_string(),
        os: os.to_string(),
        arch: arch.to_string(),
        checksum_url: None,
        checksum_md5,
        checksum_sha256,
        kind,
    })
}

#[cfg(test)]
mod test {
    #[test]
    fn test_checksum_regex() {
        let checksum_regex = regex::Regex::new(r"`(?<md5>[^`]*)`(\s+/<br\s+/>\s+`(?<sha256>[^`])`)?").unwrap();
        let caps = checksum_regex.captures("`1`").unwrap();
        assert_eq!("1", caps.name("md5").expect("").as_str());
        let caps = checksum_regex.captures("`1` /<br /> `2`").unwrap();
        assert_eq!("1", caps.name("md5").expect("").as_str());
        assert_eq!("2", caps.name("sha256").expect("").as_str());
    }
}
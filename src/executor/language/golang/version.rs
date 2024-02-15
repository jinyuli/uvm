use super::html::{
    KEY_ARCH, KEY_CHECKSUM, KEY_FILE_NAME, KEY_KIND, KEY_OS, KEY_SIZE, KEY_TYPE, KEY_URL,
};
use crate::tool::logger::debug;
use semver::{BuildMetadata, Prerelease, Version};
use std::{cmp::Ordering, collections::HashMap};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct GolangVersion {
    pub version: String,
    pub sem_version: Version,
    pub packages: Vec<GolangPackage>,
}

impl GolangVersion {
    pub fn from_map(version: String, pkgs: Vec<HashMap<&str, String>>) -> Option<Self> {
        parse_semver(version.as_str()).map(|v| GolangVersion {
            version: version.clone(),
            sem_version: v,
            packages: pkgs.into_iter().map(GolangPackage::from_map).collect(),
        })
    }

    pub fn is_match(&self, filter: &regex::Regex) -> bool {
        for package in &self.packages {
            let file_name = &package.file_name;
            if filter.is_match(file_name) {
                return true;
            }
        }
        false
    }

    pub fn has_matched_package(&self, os: &str, arch: &str) -> bool {
        debug!("has mached package with os({}) and arch({})", os, arch);
        for package in &self.packages {
            debug!("test package {:?}", package);
            if package.arch.to_lowercase() == arch
                && package.os.to_lowercase() == os
                && package.kind.to_lowercase() == "archive"
            {
                return true;
            }
        }
        false
    }

    pub fn get_matched_package(&self, os: &str, arch: &str) -> Option<&GolangPackage> {
        self.packages.iter().find(|v| {
            v.arch.to_lowercase() == arch
                && v.os.to_lowercase() == os
                && v.kind.to_lowercase() == "archive"
        })
    }
}

impl Ord for GolangVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd<GolangVersion> for GolangVersion {
    fn partial_cmp(&self, other: &GolangVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct GolangPackage {
    version_type: String,
    pub file_name: String,
    os: String,
    kind: String,
    arch: String,
    pub checksum: String,
    size: String,
    pub url: String,
}

impl GolangPackage {
    fn from_map(map: HashMap<&str, String>) -> Self {
        GolangPackage {
            version_type: map[KEY_TYPE].clone(),
            file_name: map[KEY_FILE_NAME].clone(),
            os: map[KEY_OS].clone(),
            kind: map[KEY_KIND].clone(),
            arch: map[KEY_ARCH].clone(),
            checksum: map[KEY_CHECKSUM].clone(),
            size: map[KEY_SIZE].clone(),
            url: map[KEY_URL].clone(),
        }
    }
}

pub fn parse_semver(version: &str) -> Option<Version> {
    let version_re = regex::Regex::new(r"(\d+)\.(\d+)(\.(\d+))?(.*)").unwrap();
    if let Some(caps) = version_re.captures(version) {
        let major: u64 = caps[1].parse().unwrap();
        let minor: u64 = caps[2].parse().unwrap();
        let patch: u64 = if let Some(v) = caps.get(4) {
            v.as_str().parse().unwrap()
            // u64::from_str_radix(v.as_str(), 10).unwrap()
        } else {
            0
        };
        let pre = if let Some(v) = caps.get(5) {
            v.as_str()
        } else {
            ""
        };
        Some(Version {
            major,
            minor,
            patch,
            pre: Prerelease::new(pre).unwrap(),
            build: BuildMetadata::EMPTY,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::parse_semver;
    use semver::{BuildMetadata, Prerelease, Version};

    #[test]
    fn test_parse_semver() {
        assert_eq!(
            Version {
                major: 1,
                minor: 1,
                patch: 0,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY
            },
            parse_semver("1.1.0").expect("wrong")
        );
        assert_eq!(
            Version {
                major: 1,
                minor: 1,
                patch: 0,
                pre: Prerelease::new("rc1").unwrap(),
                build: BuildMetadata::EMPTY
            },
            parse_semver("1.1.0rc1").expect("wrong")
        );
        assert_eq!(
            Version {
                major: 1,
                minor: 1,
                patch: 0,
                pre: Prerelease::new("rc1").unwrap(),
                build: BuildMetadata::EMPTY
            },
            parse_semver("1.1rc1").expect("wrong")
        );
    }
}

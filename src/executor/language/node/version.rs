use super::html::VersionItem;
use semver::Version;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq)]
pub struct NodeVersion {
    pub version: String,
    pub sem_version: Version,
    pub npm_version: Option<String>,
    pub lts: bool,
    pub packages: Vec<NodePackage>,
}

impl NodeVersion {
    pub fn from(version_item: &VersionItem) -> Option<Self> {
        if let Ok(sem_version) = Version::parse(&version_item.version[1..]) {
            let packages: Vec<NodePackage> = version_item
                .files
                .iter()
                .map(|s| s.split('-').collect::<Vec<&str>>())
                .filter(|a| a.len() > 1)
                .map(|arr| {
                    let kind = if arr.len() < 3 {
                        PackageKind::None
                    } else {
                        PackageKind::from_str(arr[2])
                    };
                    NodePackage {
                        os: arr[0].to_string(),
                        arch: arr[1].to_string(),
                        kind,
                    }
                })
                .collect();
            Some(NodeVersion {
                version: version_item.version.clone(),
                sem_version,
                npm_version: version_item.npm.clone(),
                lts: version_item.is_lts(),
                packages,
            })
        } else {
            None
        }
    }

    pub fn is_match(&self, filter: &regex::Regex) -> bool {
        filter.is_match(&self.version)
    }

    pub fn has_matched_package(&self, os: &str, arch: &str) -> bool {
        for package in &self.packages {
            if package.arch.to_lowercase() == arch
                && package.os.to_lowercase() == os
                && package.kind.is_archive()
            {
                return true;
            }
        }
        false
    }

    pub fn get_matched_package(&self, os: &str, arch: &str) -> Option<&NodePackage> {
        self.packages.iter().find(|package| {
            package.arch.to_lowercase() == arch
                && package.os.to_lowercase() == os
                && package.kind.is_archive()
        })
    }
}

impl Ord for NodeVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sem_version.cmp(&other.sem_version)
    }
}

impl PartialOrd<NodeVersion> for NodeVersion {
    fn partial_cmp(&self, other: &NodeVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NodePackage {
    pub os: String,
    pub arch: String,
    pub kind: PackageKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PackageKind {
    Tar,
    Pkg,
    Zip7z,
    Zip,
    Msi,
    Exe,
    None,
}

impl PackageKind {
    fn from_str(k: &str) -> Self {
        match k {
            "7z" => Self::Zip7z,
            "exe" => Self::Exe,
            "msi" => Self::Msi,
            "zip" => Self::Zip,
            "tar" => Self::Tar,
            "pkg" => Self::Pkg,
            _ => Self::None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Exe => "exe",
            Self::Msi => "msi",
            Self::Zip => "zip",
            Self::Zip7z => "7z",
            Self::Tar => "tar.gz",
            Self::Pkg => "pkg",
            _ => "tar.gz",
        }
    }

    fn is_archive(&self) -> bool {
        *self == Self::Zip || *self == Self::Zip7z || *self == Self::Tar
    }
}

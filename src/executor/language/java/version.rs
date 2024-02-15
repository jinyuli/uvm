use super::vendor::{Vendor, CORRETTO, OPENJDK};
use semver::{BuildMetadata, Prerelease, Version};
use std::cmp::Ordering;

/// there are three kind of versions:
///   * parsed from web page:
///         * may not follow semver
///   * used in the program
///         * no matter where the version comes from, must be converted to a standard semver
///   * displayed to user
///         * it's better to use the one from web page
#[derive(Debug, PartialEq, Eq)]
pub struct JavaVersion {
    pub vendor: &'static Vendor,
    /// version string parsed from web page
    /// this is used to display
    pub version: String,
    pub sem_version: Version,
    pub packages: Vec<JavaPackage>,
}

impl JavaVersion {
    pub fn from_corretto(packages: Vec<JavaPackage>) -> Option<Self> {
        let version = packages[0].version.clone();
        let version_str = match CORRETTO.parse_version(&version) {
            Some(v) => v,
            None => return None,
        };
        parse_semver(version_str.as_str()).map(|v| JavaVersion {
            vendor: &CORRETTO,
            version,
            sem_version: v,
            packages,
        })
    }

    pub fn from_openjdk(packages: Vec<JavaPackage>) -> Option<Self> {
        let version = packages[0].version.clone();
        let version_str = match OPENJDK.parse_version(&version) {
            Some(v) => v,
            None => return None,
        };
        parse_semver(version.as_str()).map(|v| JavaVersion {
            vendor: &OPENJDK,
            version: version_str,
            sem_version: v,
            packages,
        })
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

    pub fn get_matched_package(&self, os: &str, arch: &str) -> Option<&JavaPackage> {
        self.packages.iter().find(|package| {
            package.arch.to_lowercase() == arch
                && package.os.to_lowercase() == os
                && package.kind.is_archive()
        })
    }

    pub fn get_display_name(&self) -> String {
        format!(
            "{}-{}",
            self.vendor.name,
            self.version
        )
    }
}

impl Ord for JavaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sem_version.cmp(&other.sem_version)
    }
}

impl PartialOrd<JavaVersion> for JavaVersion {
    fn partial_cmp(&self, other: &JavaVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct JavaPackage {
    pub version: String,
    pub url: String,
    pub os: String,
    pub arch: String,
    pub kind: PackageKind,
    pub checksum_url: Option<String>,
    pub checksum_md5: Option<String>,
    pub checksum_sha256: Option<String>,
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
    pub fn from_str(k: &str) -> Self {
        match k {
            "7z" => Self::Zip7z,
            "exe" => Self::Exe,
            "msi" => Self::Msi,
            "zip" => Self::Zip,
            "tar.gz" => Self::Tar,
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

fn parse_semver(version: &str) -> Option<Version> {
    let version_re =
        regex::Regex::new(r"(\d+)(\.(\d+))?(\.(\d+))?(-([\w.]+))?(\+([\w.]+))?").unwrap();
    if let Some(caps) = version_re.captures(version) {
        let major: u64 = caps[1].parse().unwrap();
        let minor: u64 = if let Some(v) = caps.get(3) {
            v.as_str().parse().unwrap()
        } else {
            0
        };
        let patch: u64 = if let Some(v) = caps.get(5) {
            v.as_str().parse().unwrap()
        } else {
            0
        };
        let pre = if let Some(v) = caps.get(7) {
            match Prerelease::new(v.as_str()) {
                Ok(p) => p,
                Err(_) => Prerelease::EMPTY,
            }
        } else {
            Prerelease::EMPTY
        };
        let build = if let Some(v) = caps.get(9) {
            match BuildMetadata::new(v.as_str()) {
                Ok(p) => p,
                Err(_) => BuildMetadata::EMPTY,
            }
        } else {
            BuildMetadata::EMPTY
        };

        Some(Version {
            major,
            minor,
            patch,
            pre,
            build,
        })
    } else {
        None
    }
}

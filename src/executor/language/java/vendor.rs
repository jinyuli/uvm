use super::version::JavaVersion;
use super::openjdk::parse_openjdk;
use super::corretto::parse_corretto;
use super::super::language_executor::Result;

const NAME_OPENJDK: &str = "openjdk";
const NAME_CORRETTO: &str = "corretto";

pub static OPENJDK: Vendor = Vendor { name: NAME_OPENJDK };
pub static CORRETTO: Vendor = Vendor { name: NAME_CORRETTO };

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Vendor {
    pub name: &'static str,
}

impl Vendor {
    pub fn from_str(s: &str) -> Option<&Self> {
        match s {
            NAME_OPENJDK => Some(&OPENJDK),
            NAME_CORRETTO => Some(&CORRETTO),
            _ => None,
        }
    }

    pub fn trim(&self, s: &str) -> String {
        if s.starts_with(self.name) {
            s[self.name.len()+1..s.len()].to_string()
        } else {
            s.to_string()
        }
    }

    /// This is used to parse displayed version, as it maybe not a valid semver.
    pub fn parse_version(&self, s: &str) -> Option<String> {
        // remove the `vendor-` prefix
        let version = if s.starts_with(self.name) {
            &s[self.name.len()+1..s.len()]
        } else {
            s
        };
        match self.name {
            NAME_CORRETTO => {
                let version_re = regex::Regex::new(r"\d+\.\d+\.\d+(\.(?<build>.*))?").unwrap();
                match version_re.captures(version) {
                    Some(caps) => {
                        match caps.name("build") {
                            Some(build) => {
                                let build_str = build.as_str();
                                Some(format!("{}+{}", &version[0..(version.len() - build_str.len() - 1)], build_str))
                            },
                            None => Some(version.to_string()),
                        }
                    },
                    None => None,
                }
            },
            NAME_OPENJDK => Some(version.to_string()),
            _ => None,
        }
    }

    pub async fn get_versions(&self, proxy: &Option<String>) -> Result<Option<Vec<JavaVersion>>> {
        match self.name {
            NAME_OPENJDK => {
                let result = parse_openjdk(proxy).await?;
                Ok(Some(result))
            },
            NAME_CORRETTO => {
                let result = parse_corretto().await?;
                Ok(Some(result))
            },
            _ => Ok(None)
        }
    }
}

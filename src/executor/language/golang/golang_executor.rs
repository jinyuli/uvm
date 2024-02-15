use super::super::language_executor::{
    format_semver, ExecutorContext, GeneralLanguageContext, LanguageError, LanguageExecutor,
    LanguageVersion, Result,
};
use super::html;
use super::script::generate_scripts;
use super::version::GolangVersion;
use crate::executor::language::language_executor::InstallResult;
use crate::tool::checksum;
use crate::tool::logger::{debug, error};
use crate::tool::{fs, http, SupportedLanguage, GO};
use semver::{Version, VersionReq};
use std::collections::HashSet;
use std::fs::{remove_dir_all, remove_file, rename};
use std::path::Path;
use tokio::runtime;

static OFFICIAL_URL: &str = "https://go.dev/dl/";

type GoContext<'a> = ExecutorContext<'a, GeneralLanguageContext>;
pub struct GolangExecutor {
    language: &'static SupportedLanguage,
    rt: runtime::Runtime,
}

impl GolangExecutor {
    pub fn new() -> Self {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        GolangExecutor { language: GO, rt }
    }
}

impl<'a> LanguageExecutor<'a, GeneralLanguageContext> for GolangExecutor {
    fn name(&self) -> &str {
        self.language.name
    }

    fn get_env_hint(&self, current_dir: &Path) -> String {
        let bin_dir = current_dir.join("current").join("bin");
        let go_path_dir = current_dir.join("go_path");
        if cfg!(target_os="windows") {
            format!(r###"please add `{0:?}` to your path and set GOPATH to `{1:?}`"###, bin_dir, go_path_dir)
        } else {
            format!(r###"please add `{0:?}` to your path.
            for example:
            export PATH="{0:?}:$PATH"
            and set GOPATH
            export GOPATH="{1:?}"
            "###, bin_dir, go_path_dir)
        }
    }

    fn list(&self, local_only: bool, context: &'a GoContext) -> Result<Vec<LanguageVersion>> {
        let installed_version_map = self.get_installed_versions(context)?;
        if local_only {
            let installed_versions: Vec<_> = installed_version_map
                .into_iter()
                .map(|(k, v)| LanguageVersion {
                    version: k,
                    installed: true,
                    inuse: v,
                })
                .collect();
            return Ok(installed_versions);
        }
        let version_maps = self
            .rt
            .block_on(html::parse_golang_official(OFFICIAL_URL, &context.proxy))?;
        let versions: Vec<_> = version_maps
            .into_iter()
            .filter_map(|(k, v)| GolangVersion::from_map(k, v))
            .collect();
        let filter = match context.filter {
            Some(v) => v,
            None => "",
        };

        let filter_re = regex::RegexBuilder::new(&regex::escape(filter))
            .case_insensitive(true)
            .build()
            .unwrap();

        let mut set = HashSet::new();
        for version in versions {
            if filter.is_empty() || version.is_match(&filter_re) {
                set.insert(version.sem_version.clone());
            }
        }

        let mut result: Vec<Version> = set.into_iter().collect();
        result.sort();
        let returned = result
            .into_iter()
            .map(|g| -> LanguageVersion {
                let version_str = format_semver(&g);
                let installed_version = installed_version_map.get(&version_str);
                LanguageVersion {
                    version: version_str,
                    installed: installed_version.is_some(),
                    inuse: installed_version.is_some_and(|v| *v),
                }
            })
            .collect();

        Ok(returned)
    }

    fn install(&self, version: String, context: &'a GoContext) -> Result<InstallResult> {
        let req = match VersionReq::parse(&version) {
            Ok(r) => r,
            Err(e) => {
                return Err(LanguageError::GeneralString(format!("cannot parse the given version({}): {}", version, e)));
            },
        };
        let version_maps = self
            .rt
            .block_on(html::parse_golang_official(OFFICIAL_URL, &context.proxy))?;
        let mut versions: Vec<_> = version_maps
            .into_iter()
            .filter_map(|(k, v)| GolangVersion::from_map(k, v))
            .collect();
        versions.sort();

        let sys_arch = std::env::consts::ARCH.to_lowercase();
        let arch = convert_to_go_arch(sys_arch.as_str());
        let sys_os = std::env::consts::OS.to_lowercase();
        let os = convert_to_go_os(sys_os.as_str());
        debug!("current arch:{}, os:{}", arch, os);
        let mut matched_version: Option<GolangVersion> = None;
        let mut exact_matched_version: Option<GolangVersion> = None;
        debug!("version request {}", req);
        let go_ver = format!("go{}", version);
        for item in versions.into_iter().rev() {
            let sem_version = &item.sem_version;
            // debug!("test with version: {}", version);
            if item.version == go_ver {
                exact_matched_version = Some(item);
                break;
            }
            if exact_matched_version.is_none()
                && matched_version.is_none()
                && req.matches(sem_version)
            {
                matched_version = Some(item);
            }
        }
        if exact_matched_version.is_some() {
            matched_version = exact_matched_version;
        }
        debug!("found matched version: {:?}", matched_version);

        match matched_version {
            Some(version) => {
                if version.has_matched_package(os.as_str(), arch) {
                    let installed_versions_result = self.get_installed_versions(context);
                    match installed_versions_result {
                        Ok(installed_versions) => {
                            let version_str = format_semver(&version.sem_version);
                            if installed_versions.contains_key(&version_str) {
                                debug!("version {} has been installed", version_str);
                                Ok(InstallResult::VersionInstalled)
                            } else {
                                let package = match version
                                    .get_matched_package(os.as_str(), arch) {
                                        Some(p) => p,
                                        None => return Err(LanguageError::NoMatchedArchOrOS(sys_arch, sys_os)),
                                    };
                                let download_dir = context.language_dir.get_tmp_dir();
                                let downloaded_file = download_dir.join(&package.file_name);
                                let mut need_download = true;
                                if downloaded_file.exists() && downloaded_file.is_file() {
                                    // TODO should validate the file other than download it again
                                    if package.checksum.is_empty() {
                                        debug!("file to download exists and cannot verify the file, delete it");
                                        remove_file(&downloaded_file)?;
                                    } else if checksum::verify(checksum::ChecksumMethod::Sha256, &downloaded_file, &package.checksum)?
                                    {
                                        need_download = true;
                                        debug!("file to download exists, use it");
                                    } else {
                                        debug!("file to download exists and the file's checksum is wrong, delete it");
                                        remove_file(&downloaded_file)?;
                                    }
                                }
                                let versions = context.language_dir.get_versions_dir();
                                let archived_dir = versions.join(version_str);
                                if need_download {
                                    let mut url = package.url.clone();
                                    if url.starts_with('/') {
                                        url = format!("https://go.dev{}", url);
                                    }
                                    debug!("download file({}) to {:?}", &url, &downloaded_file);
                                    self.rt.block_on(http::download_file(
                                        &url,
                                        &downloaded_file,
                                        &context.proxy,
                                    ))?;
                                    if !checksum::verify(checksum::ChecksumMethod::Sha256, &downloaded_file, &package.checksum)? {
                                        error!("failed to verify the downloaded file, delete it");
                                        remove_file(&downloaded_file)?;
                                        return Err(LanguageError::FailedToVerify());
                                    }
                                }

                                // TODO here it assumes that all go archive top folder is `go`
                                let go_dir = versions.join("go");
                                if go_dir.exists() && go_dir.is_dir() {
                                    remove_dir_all(&go_dir)?;
                                }
                                debug!("unzip file to {:?}", &go_dir);
                                fs::decompress(&downloaded_file, versions)?;
                                debug!("rename go to version");
                                rename(&go_dir, &archived_dir)?;

                                let mut need_hint = false;
                                if !context.language_context.as_ref().is_some_and(|c| c.no_use) {
                                    debug!("make symbol link");
                                    let current_dir = context.language_dir.get_current_dir();
                                    if current_dir.exists() {
                                        debug!("delete current symbol link");
                                        fs::remove_link(current_dir)?;
                                    } else {
                                        need_hint = true;
                                    }
                                    fs::make_link(current_dir, &archived_dir)?;
                                }
                                remove_file(downloaded_file)?;
                                if need_hint {
                                    Ok(InstallResult::SuccessNeedHint)
                                } else {
                                    Ok(InstallResult::Success)
                                }
                            }
                        }
                        Err(_) => Err(LanguageError::FailedToReadFS()),
                    }
                } else {
                    Err(LanguageError::NoMatchedArchOrOS(
                        arch.to_string(),
                        os.to_string(),
                    ))
                }
            }
            None => Err(LanguageError::NoSuchVersion(version)),
        }
    }

    fn post_venv(&self, dir: &Path) -> Result<()> {
        generate_scripts(dir)?;
        Ok(())
    }
}

fn convert_to_go_os(os: &str) -> String {
    os.to_lowercase()
}

fn convert_to_go_arch(arch: &str) -> &str {
    match arch {
        "x86_64" => "x86-64",
        "arm" => "armv6",
        "aarch64" => "arm64",
        "loongarch64" => "loong64",
        "powerpc" => "ppc",
        "powerpc64" => "ppc64",
        _ => arch,
    }
}

#[cfg(test)]
mod test {
    use semver::VersionReq;

    use super::super::version::parse_semver;

    #[test]
    fn test_file_regex() {
        let file_name_re = regex::Regex::new(r"go(?<version>\d+\.\d+(\.\d+)?([^.]+)?)\.(?<middle>((\w+)-(\w+))|src)\.(?<pkg>msi|pkg|tar\.gz)(\..+)?").unwrap();
        let caps = file_name_re.captures("go1.10.1.darwin-amd64.pkg").unwrap();
        assert_eq!("1.10.1", caps.name("version").expect("no version").as_str());
        assert_eq!(
            "darwin-amd64",
            caps.name("middle").expect("no version").as_str()
        );
        assert_eq!("pkg", caps.name("pkg").expect("no version").as_str());
        let caps = file_name_re.captures("go1.10.1.src.tar.gz").unwrap();
        assert_eq!("1.10.1", caps.name("version").expect("no version").as_str());
        assert_eq!("src", caps.name("middle").expect("no version").as_str());
        assert_eq!("tar.gz", caps.name("pkg").expect("no version").as_str());
        let caps = file_name_re
            .captures("go1.10beta1.linux-386.tar.gz")
            .unwrap();
        assert_eq!(
            "1.10beta1",
            caps.name("version").expect("no version").as_str()
        );
        assert_eq!(
            "linux-386",
            caps.name("middle").expect("no version").as_str()
        );
        assert_eq!("tar.gz", caps.name("pkg").expect("no version").as_str());
        let caps = file_name_re
            .captures("go1.10beta1.linux-386.tar.gz.sha256")
            .unwrap();
        assert_eq!(
            "1.10beta1",
            caps.name("version").expect("no version").as_str()
        );
        assert_eq!(
            "linux-386",
            caps.name("middle").expect("no version").as_str()
        );
        assert_eq!("tar.gz", caps.name("pkg").expect("no version").as_str());
    }

    #[test]
    fn test_version_request() {
        let req = VersionReq::parse("1.20.1").unwrap();
        let v1 = parse_semver(&String::from("1.21.6")).expect("");
        assert!(req.matches(&v1));
        let v2 = parse_semver(&String::from("1.1.6")).expect("");
        assert!(!req.matches(&v2));
    }
}

use super::super::language_executor::{
    format_semver, ExecutorContext, LanguageError, LanguageExecutor, LanguageVersion, Result,
};
use super::html::{parse_node_official, VersionItem};
use super::script::generate_scripts;
use super::version::NodeVersion;
use crate::executor::language::language_executor::{GeneralLanguageContext, InstallResult};
use crate::tool::checksum;
use crate::tool::logger::{debug, error, info};
use crate::tool::{fs, http, SupportedLanguage, NODE};
use semver::VersionReq;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::{remove_dir_all, remove_file, rename};
use std::path::Path;
use tokio::runtime;

static OFFICIAL_URL: &str = "https://nodejs.org/dist/index.json";

pub struct NodeExecutor {
    language: &'static SupportedLanguage,
    rt: runtime::Runtime,
}

impl NodeExecutor {
    pub fn new() -> Self {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        NodeExecutor { language: NODE, rt }
    }

    fn verify_file(&self, file_path: &Path, checksum: &str) -> Result<bool> {
        let result = checksum::verify(checksum::ChecksumMethod::Sha256, file_path, checksum)?;
        Ok(result)
    }

    fn get_checksum(
        &self,
        url: &String,
        file_name: &String,
        proxy: &Option<String>,
    ) -> Result<Option<String>> {
        debug!("get checksum from {} for {}", url, file_name);
        let bs = self.rt.block_on(http::download_html(url, proxy))?;
        debug!("checksum file content: {}", &bs);
        let digests = bs
            .lines()
            .map(|line| {
                line.split(&[' ', '\t'])
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|p| p.len() > 1)
            .map(|v| (v[1], v[0]))
            .collect::<HashMap<_, _>>();

        debug!("checksum map: {:?}", &digests);
        match digests.get(file_name.as_str()) {
            Some(v) => Ok(Some(v.to_string())),
            None => Ok(None),
        }
    }
}

type NodeContext<'a> = ExecutorContext<'a, GeneralLanguageContext>;

impl<'a> LanguageExecutor<'a, GeneralLanguageContext> for NodeExecutor {
    fn name(&self) -> &str {
        self.language.name
    }

    fn get_env_hint(&self, current_dir: &Path) -> String {
        let current_dir = current_dir.join("current");
        if cfg!(target_os="windows") {
            format!(r###"please add `{0:?}` to your path."###, current_dir)
        } else {
            format!(r###"please add `{0:?}` to your path.
            for example:
            export PATH="{0:?}:$PATH"
            "###, current_dir.as_os_str())
        }
    }

    fn list(&self, local_only: bool, context: &'a NodeContext<'a>) -> Result<Vec<LanguageVersion>> {
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
        let version_items: Vec<VersionItem> = match self
            .rt
            .block_on(parse_node_official(OFFICIAL_URL, &context.proxy))
        {
            Ok(vs) => vs,
            Err(e) => {
                error!("failed to parse node version list: {}", e);
                return Err(LanguageError::Html());
            }
        };

        // println!("versions: {:?}", version_items);
        let filter = match context.filter {
            Some(v) => v,
            None => "",
        };
        let filter_re = regex::RegexBuilder::new(&regex::escape(filter))
            .case_insensitive(true)
            .build()
            .unwrap();

        let mut versions: Vec<NodeVersion> = version_items
            .iter()
            .map(NodeVersion::from)
            .filter(|o| {
                o.as_ref()
                    .is_some_and(|f| filter.is_empty() || f.is_match(&filter_re))
            })
            .map(|o| o.expect(""))
            .collect();
        versions.sort();

        let returned: Vec<LanguageVersion> = versions
            .into_iter()
            .map(|g| -> LanguageVersion {
                let version_str = format_version(&g);
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

    fn install(&self, version: String, context: &'a NodeContext<'a>) -> Result<InstallResult> {
        let req = match VersionReq::parse(version.as_str()) {
            Ok(r) => r,
            Err(e) => {
                return Err(LanguageError::GeneralString(format!("cannot parse the given version({}): {}", version, e)));
            },
        };
        let version_items: Vec<VersionItem> = match self
            .rt
            .block_on(parse_node_official(OFFICIAL_URL, &context.proxy))
        {
            Ok(vs) => vs,
            Err(e) => {
                error!("failed to parse node version list: {}", e);
                return Err(LanguageError::Html());
            }
        };

        let mut versions: Vec<NodeVersion> =
            version_items.iter().filter_map(NodeVersion::from).collect();
        versions.sort();

        let sys_arch = std::env::consts::ARCH.to_lowercase();
        let arch = convert_to_node_arch(sys_arch.as_str());
        let sys_os = std::env::consts::OS.to_lowercase();
        let os = convert_to_node_os(sys_os.as_str());
        debug!("current arch:{}, os:{}", arch, os);
        let mut matched_version: Option<NodeVersion> = None;
        let mut exact_matched_version: Option<NodeVersion> = None;
        debug!("version request {}", req);
        let node_ver = format!("v{}", version);
        for item in versions.into_iter().rev() {
            let sem_version = &item.sem_version;
            // debug!("test with version: {}", version);
            if item.version == node_ver {
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
                if version.has_matched_package(os, arch) {
                    let installed_versions_result = self.get_installed_versions(context);
                    match installed_versions_result {
                        Ok(installed_versions) => {
                            let version_str = format_semver(&version.sem_version);
                            if installed_versions.contains_key(&version_str) {
                                info!("version {} has been installed", &version_str);
                                Err(LanguageError::VersionInstalled())
                            } else {
                                let package = version
                                    .get_matched_package(os, arch)
                                    .expect("should have a matched package");
                                let download_dir = context.language_dir.get_tmp_dir();
                                let file_name = format!(
                                    "node-v{}-{}-{}",
                                    version_str,
                                    convert_to_file_name_os(os),
                                    arch
                                );
                                let archive_name =
                                    format!("{}.{}", file_name, package.kind.to_str());
                                let downloaded_file = download_dir.join(&archive_name);

                                let checksum_url = format!(
                                    "https://nodejs.org/dist/v{}/SHASUMS256.txt",
                                    &version_str
                                );
                                let checksum = self.get_checksum(
                                    &checksum_url,
                                    &archive_name,
                                    &context.proxy,
                                )?;
                                let mut need_download = true;
                                if downloaded_file.exists() && downloaded_file.is_file() {
                                    match &checksum {
                                        Some(c) => {
                                            if self.verify_file(&downloaded_file, c)? {
                                                need_download = false;
                                                debug!("file to download exists, use it");
                                            } else {
                                                debug!("file to download exists and the file's checksum is wrong, delete it");
                                                remove_file(&downloaded_file)?;
                                            }
                                        }
                                        None => {
                                            debug!("file to download exists and cannot verify the file, delete it");
                                            remove_file(&downloaded_file)?;
                                        }
                                    }
                                }
                                let versions = context.language_dir.get_versions_dir();
                                let archived_dir = versions.join(&version_str);
                                if need_download {
                                    let url = format!(
                                        "https://nodejs.org/dist/v{}/{}",
                                        &version_str, &archive_name
                                    );
                                    debug!("download file({}) to {:?}", &url, &downloaded_file);
                                    self.rt.block_on(http::download_file(
                                        &url,
                                        &downloaded_file,
                                        &context.proxy,
                                    ))?;
                                    match &checksum {
                                        Some(c) => {
                                            if !self.verify_file(&downloaded_file, c)? {
                                                error!("failed to verify the downloaded file");
                                                return Err(LanguageError::FailedToVerify());
                                            }
                                        }
                                        None => {
                                            debug!("cannot find checksum, use the file downloaded");
                                        }
                                    }
                                }

                                let node_dir = versions.join(file_name);
                                if node_dir.exists() && node_dir.is_dir() {
                                    remove_dir_all(&node_dir)?;
                                }
                                debug!("unzip file to {:?}", &node_dir);
                                let dest_folder = fs::decompress(&downloaded_file, versions)?;
                                debug!("unzipped file name {:?}", dest_folder);

                                debug!("rename go to version");
                                rename(&node_dir, &archived_dir)?;

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

fn convert_to_file_name_os(os: &str) -> &str {
    match os.to_lowercase().as_str() {
        "win" => "win",
        "osx" => "darwin",
        "aix" => "aix",
        "sunos" => "sunos",
        _ => "linux",
    }
}

fn convert_to_node_os(os: &str) -> &str {
    match os.to_lowercase().as_str() {
        "windows" => "win",
        "macos" => "osx",
        "aix" => "aix",
        "sunos" => "sunos",
        _ => "linux",
    }
}

fn convert_to_node_arch(arch: &str) -> &str {
    match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "loongarch64" => "loong64",
        "powerpc" => "ppc",
        "powerpc64" => "ppc64",
        "powerpc64le" => "ppc64le",
        _ => arch,
    }
}
pub fn format_version(version: &NodeVersion) -> String {
    let sem_version = &version.sem_version;
    let mut base = String::new();
    let _ = write!(
        base,
        "{}.{}.{}",
        sem_version.major, sem_version.minor, sem_version.patch
    );
    if !sem_version.pre.is_empty() {
        let _ = write!(base, "-{}", sem_version.pre.as_str());
    }
    if !sem_version.build.is_empty() {
        let _ = write!(base, "+{}", sem_version.build.as_str());
    }
    if version.lts {
        let _ = write!(base, " LTS");
    }
    base
}

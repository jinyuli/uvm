use super::super::language_executor::{
    ExecutorContext, LanguageContext, LanguageError, LanguageExecutor, LanguageVersion, Result,
};
use super::scripts::generate_scripts;
use super::vendor::Vendor;
use super::version::JavaVersion;
use crate::executor::language::language_executor::InstallResult;
use crate::tool::checksum;
use crate::tool::logger::{debug, error, info};
use crate::tool::{fs, http, SupportedLanguage, JAVA};
use semver::VersionReq;
use std::fs::{remove_file, rename};
use std::path::Path;
use tokio::runtime;

pub struct JavaLanguageContext {
    pub vendor: String,
    /// used in `install` command, whether to use installed version instantly.
    pub no_use: bool
}

impl LanguageContext for JavaLanguageContext {}

type JavaContext<'a> = ExecutorContext<'a, JavaLanguageContext>;

pub struct JavaExecutor {
    language: &'static SupportedLanguage,
    rt: runtime::Runtime,
}

impl<'a> JavaExecutor {
    pub fn new() -> Self {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        JavaExecutor { language: JAVA, rt }
    }

    fn find_version_install(
        &self,
        version: String,
        context: &'a JavaContext<'a>,
        matched_version: Option<JavaVersion>,
        os: &str,
        arch: &str,
    ) -> Result<JavaVersion> {
        match matched_version {
            Some(version) => {
                if version.has_matched_package(os, arch) {
                    let installed_versions_result = self.get_installed_versions(context);
                    match installed_versions_result {
                        Ok(installed_versions) => {
                            let version_str = version.get_display_name();
                            if installed_versions.contains_key(&version_str) {
                                info!("version {} has been installed", &version_str);
                                Err(LanguageError::VersionInstalled())
                            } else {
                                Ok(version)
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
}

impl<'a> LanguageExecutor<'a, JavaLanguageContext> for JavaExecutor {
    fn name(&self) -> &str {
        self.language.name
    }

    fn get_env_hint(&self, current_dir: &Path) -> String {
        let current_dir = current_dir.join("current");
        let bin_dir = current_dir.join("bin");
        if cfg!(target_os="windows") {
            format!(r###"please add `{0:?}` to your path and set JAVA_HOME to `{1:?}`"###, bin_dir, current_dir)
        } else {
            format!(r###"please add `{0:?}` to your path.
            for example:
            export PATH="{0:?}:$PATH"
            and set JAVA_HOME
            export JAVA_HOME="{1:?}"
            "###, bin_dir, current_dir)
        }
    }

    fn list(&self, local_only: bool, context: &'a JavaContext<'a>) -> Result<Vec<LanguageVersion>> {
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

        let language_context = match context.language_context.as_ref() {
            Some(l) => l,
            None => {
                return Err(LanguageError::General("Java vendor is required to list remote versions, valid vendors include openjdk, corretto"));
            }
        };

        let vendor_arg = &language_context.vendor.to_lowercase();
        let vendor = match Vendor::from_str(vendor_arg.as_str()) {
            Some(v) => v,
            None => {
                return Err(LanguageError::General("Java vendor is required to list remote versions, valid vendors include openjdk, corretto"))
            },
        };

        let version_items: Vec<JavaVersion> =
            match self.rt.block_on(vendor.get_versions(&context.proxy)) {
                Ok(vs) => match vs {
                    Some(v) => v,
                    None => {
                        error!("Unsupported vendor");
                        return Err(LanguageError::Html());
                    }
                },
                Err(e) => {
                    error!("failed to parse node version list: {}", e);
                    return Err(LanguageError::Html());
                }
            };

        let filter = match context.filter {
            Some(v) => v,
            None => "",
        };
        let filter_re = regex::RegexBuilder::new(&regex::escape(filter))
            .case_insensitive(true)
            .build()
            .unwrap();

        let mut versions: Vec<JavaVersion> = version_items
            .into_iter()
            .filter(|o| filter.is_empty() || o.is_match(&filter_re))
            .collect();
        versions.sort();

        let returned: Vec<LanguageVersion> = versions
            .into_iter()
            .map(|g| -> LanguageVersion {
                let version_str = format_installed_version(vendor, &g);
                let installed_version = installed_version_map.get(&version_str);
                LanguageVersion {
                    version: g.version.clone(),
                    installed: installed_version.is_some(),
                    inuse: installed_version.is_some_and(|v| *v),
                }
            })
            .collect();

        Ok(returned)
    }

    fn install(&self, version: String, context: &'a JavaContext<'a>) -> Result<InstallResult> {
        let language_context = context.language_context.as_ref();
        let vendor_arg = &language_context.expect("Java vendor is required to install command, valid vendors include openjdk, corretto").vendor.to_lowercase();
        let vendor = Vendor::from_str(vendor_arg.as_str()).expect(
            "Java vendor is required to install command, valid vendors include openjdk, corretto",
        );

        let trimed_version = vendor.trim(&version);
        let parsed_version = match vendor.parse_version(&trimed_version) {
            Some(v) => v,
            None => return Err(LanguageError::GeneralString(format!("Version({}) is not valid for {}", version, vendor.name))),
        };
        let req = match VersionReq::parse(&parsed_version) {
            Ok(r) => r,
            Err(e) => {
                return Err(LanguageError::GeneralString(format!("cannot parse the given version({}): {}", version, e)));
            },
        };
        let version_items: Vec<JavaVersion> =
            match self.rt.block_on(vendor.get_versions(&context.proxy)) {
                Ok(vs) => match vs {
                    Some(v) => v,
                    None => {
                        error!("Unsupported vendor");
                        return Err(LanguageError::Html());
                    }
                },
                Err(e) => {
                    error!("failed to parse node version list: {}", e);
                    return Err(LanguageError::Html());
                }
            };

        let mut versions: Vec<JavaVersion> = version_items;
        versions.sort();

        let sys_arch = std::env::consts::ARCH.to_lowercase();
        let arch = convert_to_java_arch(sys_arch.as_str());
        let sys_os = std::env::consts::OS.to_lowercase();
        let os = convert_to_java_os(sys_os.as_str());
        debug!("current arch:{}, os:{}", arch, os);
        let mut matched_version: Option<JavaVersion> = None;
        let mut exact_matched_version: Option<JavaVersion> = None;
        debug!("version request {}", req);
        for item in versions.into_iter().rev() {
            // debug!("test with version: {}", version);
            if item.version == trimed_version {
                exact_matched_version = Some(item);
                break;
            }
            let sem_version = &item.sem_version;
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

        match self.find_version_install(version, context, matched_version, os, arch) {
            Ok(version) => {
                let version_str = version.get_display_name();
                let package = match version
                    .get_matched_package(os, arch) {
                        Some(p) => p,
                        None => return Err(LanguageError::NoMatchedArchOrOS(sys_arch, sys_os)),
                    };
                let download_dir = context.language_dir.get_tmp_dir();
                let segments: Vec<&str> = package.url.split('/').collect();
                let file_name = segments[segments.len() - 1];
                // let archive_name = format!("{}.{}", file_name, package.kind.to_str());
                let downloaded_file = download_dir.join(file_name);

                let mut checksum_method = checksum::ChecksumMethod::None;
                let mut checksum = String::new();
                // TODO checksum method may be wrong in the future, as it's parsed from html.
                if let Some(v) = package.checksum_md5.as_ref() {
                    checksum_method = checksum::ChecksumMethod::Md5;
                    checksum = v.clone();
                } else if let Some(v) = package.checksum_sha256.as_ref() {
                    checksum_method = checksum::ChecksumMethod::Sha256;
                    checksum = v.clone();
                } else if let Some(v) = package.checksum_url.as_ref() {
                    checksum_method = checksum::ChecksumMethod::Sha256;
                    checksum = self.rt.block_on(http::download_html(v, &context.proxy))?;
                }

                let mut need_download = true;
                if downloaded_file.exists() && downloaded_file.is_file() {
                    if checksum.is_empty() || checksum_method == checksum::ChecksumMethod::None {
                        debug!("file to download exists and cannot verify it, delete it");
                        remove_file(&downloaded_file)?;
                    } else if checksum::verify(
                        checksum_method.clone(),
                        &downloaded_file,
                        &checksum,
                    )? {
                        debug!("file to download exists and checksum verified, use it");
                        need_download = false;
                    } else {
                        error!("file to download exists and failed to verify it, delete it");
                        remove_file(&downloaded_file)?;
                    }
                }

                let versions = context.language_dir.get_versions_dir();
                let archived_dir = versions.join(version_str);
                if need_download {
                    debug!("download file({}) to {:?}", &package.url, &downloaded_file);
                    self.rt.block_on(http::download_file(
                        &package.url,
                        &downloaded_file,
                        &context.proxy,
                    ))?;
                    if checksum.is_empty() || checksum_method == checksum::ChecksumMethod::None {
                        debug!("no checksum with downloaded file, use it");
                    } else if !checksum::verify(checksum_method, &downloaded_file, &checksum)? {
                        error!("failed to verify download file, delete it");
                        remove_file(&downloaded_file)?;
                        return Err(LanguageError::FailedToVerify());
                    }
                }

                debug!("unzip file");
                let dest_folder = fs::decompress(&downloaded_file, versions)?;
                debug!("unzipped file name {:?}", dest_folder);

                // need to delete node dir first if it exists
                // but cannot get folder name before decompression
                // need to figure out a way to get root folder of an archive file
                // if node_dir.exists() && node_dir.is_dir() {
                //     remove_dir_all(&node_dir)?;
                // }

                let node_dir = match dest_folder {
                    Some(f) => versions.join(f),
                    None => {
                        let folder_name =
                            &file_name[0..(file_name.len() - package.kind.to_str().len())];
                        versions.join(folder_name)
                    }
                };

                debug!("rename folder {:?} to version", &node_dir);
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
            Err(err) => Err(err),
        }
    }

    fn post_venv(&self, dir: &Path) -> Result<()> {
        generate_scripts(dir)?;
        Ok(())
    }
}

fn convert_to_java_os(os: &str) -> &str {
    match os.to_lowercase().as_str() {
        "windows" => "windows",
        "macos" => "macos",
        _ => "linux",
    }
}

fn convert_to_java_arch(arch: &str) -> &str {
    match arch {
        "x86_64" => "x64",
        "loongarch64" => "loong64",
        "powerpc" => "ppc",
        "powerpc64" => "ppc64",
        "powerpc64le" => "ppc64le",
        _ => arch,
    }
}

pub fn format_installed_version(vendor: &Vendor, version: &JavaVersion) -> String {
    format!("{}-{}", vendor.name, version.version)
}

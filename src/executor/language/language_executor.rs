use crate::tool::fs::{make_link, remove_link, FSError, LanguageDir};
use crate::tool::http::HttpError;
use crate::tool::logger::debug;
use semver::Version;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::{create_dir, read_dir, read_link, remove_dir_all};
use std::path::Path;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, LanguageError>;

/// LanguageExecutor uses to execute commands for language version management.
pub trait LanguageExecutor<'a, T: LanguageContext> {
    fn name(&self) -> &str;
    fn get_env_hint(&self, current_dir: &Path) -> String;
    fn list(
        &self,
        local_only: bool,
        context: &'a ExecutorContext<'a, T>,
    ) -> Result<Vec<LanguageVersion>>;
    fn install(&self, version: String, context: &'a ExecutorContext<'a, T>) -> Result<InstallResult>;
    fn post_venv(&self, dir: &Path) -> Result<()>;

    fn unuse(&self, context: &'a ExecutorContext<'a, T>) -> Result<()> {
        let current_version = self.get_current_version(context);
        if current_version.is_some() {
            let current_dir = context.language_dir.get_current_dir();
            remove_link(current_dir)?;
        }
        Ok(())
    }

    fn select(&self, version: String, context: &'a ExecutorContext<'a, T>) -> Result<UseResult> {
        let installed_version = self.get_installed_versions(context)?;
        let mut show_hint = false;
        if installed_version.contains_key(&version) {
            let current_version = self.get_current_version(context);
            let current_dir = context.language_dir.get_current_dir();
            let versions_dir = context.language_dir.get_versions_dir();
            let version_dir = versions_dir.join(&version);
            if current_version.is_none() {
                show_hint = true;
                make_link(current_dir, &version_dir)?;
            } else if current_version.is_some_and(|v| v != version) {
                remove_link(current_dir)?;
                make_link(current_dir, &version_dir)?;
            } else {
                return Ok(UseResult::VersionAlreadyUsed);
            }
        } else {
            return Err(LanguageError::VersionNotInstalled(version));
        }
        if show_hint {
            Ok(UseResult::SuccessNeedHint)
        } else {
            Ok(UseResult::Success)
        }
    }

    fn uninstall(&self, version: String, context: &'a ExecutorContext<'a, T>) -> Result<()> {
        let current_version = self.get_current_version(context);
        if current_version.is_some_and(|v| v == version) {
            let current_dir = context.language_dir.get_current_dir();
            remove_link(current_dir)?;
        }

        let installed_version = self.get_installed_versions(context)?;
        if installed_version.contains_key(&version) {
            let versions = context.language_dir.get_versions_dir();
            let version_dir = versions.join(&version);
            remove_dir_all(version_dir)?;
        }
        Ok(())
    }

    fn venv(
        &self,
        version: String,
        dir_name: String,
        context: &'a ExecutorContext<'a, T>,
    ) -> Result<VenvResult> {
        let current_dir = std::env::current_dir()?;
        debug!("current dir is {:?}, version: {}", &current_dir, &version);
        let venv_dir = current_dir.join(dir_name);
        let installed_version = self.get_installed_versions(context)?;
        debug!("installed versions: {:?}", installed_version);
        if !installed_version.contains_key(&version) {
            return Err(LanguageError::VersionNotInstalled(version));
        }
        let need_create = if venv_dir.exists() && venv_dir.is_dir() {
            let link_dir = venv_dir.join(self.name());
            let linked_name = self.get_linked_name(&link_dir);
            if linked_name.is_some_and(|v| v == version) {
                return Ok(VenvResult::VersionAlreadyUsed);
            } else {
                true
            }
        } else {
            true
        };
        if need_create {
            if venv_dir.exists() {
                remove_dir_all(&venv_dir)?;
            }
            create_dir(&venv_dir)?;
            let link_dir = venv_dir.join(self.name());
            let versions_dir = context.language_dir.get_versions_dir();
            let version_dir = versions_dir.join(&version);
            make_link(&link_dir, &version_dir)?;
            self.post_venv(&venv_dir)?;
        }
        Ok(VenvResult::Success)
    }

    /// get all installed version locally.
    fn get_installed_versions(
        &self,
        context: &ExecutorContext<'a, T>,
    ) -> Result<HashMap<String, bool>> {
        let current_version = self.get_current_version(context);
        let versions_dir = context.language_dir.get_versions_dir();
        debug!("versions dir: {:?}", versions_dir);
        let mut result = HashMap::new();
        if versions_dir.exists() && versions_dir.is_dir() {
            for entry in read_dir(versions_dir)? {
                let p = entry?.path();
                if p.is_dir() {
                    let dir_name = p
                        .file_name()
                        .expect("cannot get folder name")
                        .to_str()
                        .expect("cannot convert OsStr to str");

                    result.insert(
                        dir_name.to_string(),
                        current_version.as_ref().is_some_and(|v| v == dir_name),
                    );
                }
            }
        }
        Ok(result)
    }

    /// get currently using version.
    fn get_current_version(&self, context: &ExecutorContext<'a, T>) -> Option<String> {
        let current = context.language_dir.get_current_dir();
        self.get_linked_name(current)
    }

    /// get name of the folder linked by `link_file`.
    fn get_linked_name(&self, link_file: &Path) -> Option<String> {
        if link_file.exists() && link_file.is_symlink() {
            let path_link = read_link(link_file).expect("read symblink file error");
            let file_name = path_link.file_name().expect("current symbol link error");
            let name_str = file_name.to_str().expect("wrong linked file name");
            Some(name_str.to_string())
        } else {
            None
        }
    }

    /// create a symbol link from `link_dir` to `origin_dir`.
    fn link_dir(&self, link_dir: &Path, origin_dir: &Path) -> Result<()> {
        make_link(link_dir, origin_dir)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum InstallResult {
    Success,
    SuccessNeedHint,
    VersionInstalled,
}

#[derive(Clone, Debug)]
pub enum UseResult {
    Success,
    SuccessNeedHint,
    VersionAlreadyUsed,
}

#[derive(Clone, Debug)]
pub enum VenvResult {
    Success,
    VersionAlreadyUsed,
}

/// Context for executor.
#[derive(Clone, Debug)]
pub struct ExecutorContext<'a, T: LanguageContext> {
    pub proxy: Option<String>,
    pub language_dir: &'a LanguageDir,
    pub language_context: Option<T>,
    pub filter: Option<&'a String>,
    pub arch: &'a str,
    pub os: &'a str,
}

impl<'a, T: LanguageContext> ExecutorContext<'a, T> {
    pub fn merge(&mut self, language_context: Option<T>) -> &mut Self {
        self.language_context = language_context;
        self
    }
}

pub trait LanguageContext {}

pub struct GeneralLanguageContext {
    /// used in `install` command, whether to use installed version instantly.
    pub no_use: bool
}

impl LanguageContext for GeneralLanguageContext {}

#[derive(Clone, Debug)]
pub struct LanguageVersion {
    pub version: String,
    pub installed: bool,
    pub inuse: bool,
}

pub fn format_semver(version: &Version) -> String {
    let mut base = String::new();
    let _ = write!(
        base,
        "{}.{}.{}",
        version.major, version.minor, version.patch
    );
    if !version.pre.is_empty() {
        let _ = write!(base, "-{}", version.pre.as_str());
    }
    if !version.build.is_empty() {
        let _ = write!(base, "+{}", version.build.as_str());
    }
    base
}

#[derive(Error, Debug)]
pub enum LanguageError {
    #[error("{0}")]
    Http(#[from] HttpError),
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("fs error: {0}")]
    FS(#[from] FSError),
    #[error("no matched version with {0}")]
    NoSuchVersion(String),
    #[error("no matched arch({0}) or os({1})")]
    NoMatchedArchOrOS(String, String),
    #[error("{0}")]
    Octocrab(#[from] octocrab::Error),
    #[error("failed to read local fs")]
    FailedToReadFS(),
    #[error("Version already installed")]
    VersionInstalled(),
    #[error("Version {0} has not been installed")]
    VersionNotInstalled(String),
    #[error("Failed to parse versions")]
    Html(),
    #[error("Failed to verify the file")]
    FailedToVerify(),
    #[error("{0}")]
    General(&'static str),
    #[error("{0}")]
    GeneralString(String),
}

use super::logger::debug;
use directories::UserDirs;
use flate2::read::GzDecoder;
use sevenz_rust;
use sevenz_rust::default_entry_extract_fn;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Component;
use std::path::{Path, PathBuf};
use tar::Archive;
use thiserror::Error;
use zip;

pub type Result<T> = std::result::Result<T, FSError>;

#[derive(Clone, Debug)]
pub struct AppDir {
    home_dir: PathBuf,
    log_dir: PathBuf,
    language_dirs: HashMap<String, LanguageDir>,
}

#[derive(Clone, Debug)]
pub struct LanguageDir {
    home_dir: PathBuf,
    versions_dir: PathBuf,
    current_dir: PathBuf,
    tmp_dir: PathBuf,
}

impl AppDir {
    pub fn new() -> Result<Self> {
        if let Some(user_dir) = UserDirs::new() {
            let home = user_dir.home_dir();
            let uvm_home = home.join(".uvm");
            if !uvm_home.exists() || !uvm_home.is_dir() {
                fs::create_dir(uvm_home.clone())?;
            }

            let data_path = uvm_home.join("data");
            if !data_path.exists() || !data_path.is_dir() {
                fs::create_dir(data_path.clone())?;
            }

            let log_path = uvm_home.join("log");
            if !log_path.exists() || !log_path.is_dir() {
                fs::create_dir(log_path.clone())?;
            }

            let mut language_dirs = HashMap::new();
            for name in ["go", "java", "node"] {
                let lang_dir = data_path.join(name);
                if !lang_dir.exists() || !lang_dir.is_dir() {
                    fs::create_dir(lang_dir.clone())?;
                }
                let versions_path = lang_dir.join("versions");
                if !versions_path.exists() || !versions_path.is_dir() {
                    fs::create_dir(versions_path.clone())?;
                }
                let tmp_path = lang_dir.join("tmp");
                if !tmp_path.exists() || !tmp_path.is_dir() {
                    fs::create_dir(tmp_path.clone())?;
                }
                let current_dir = lang_dir.join("current");
                if !current_dir.exists() || !current_dir.is_dir() {
                    fs::create_dir(current_dir.clone())?;
                }
                language_dirs.insert(
                    name.to_string(),
                    LanguageDir {
                        home_dir: lang_dir,
                        versions_dir: versions_path,
                        current_dir,
                        tmp_dir: tmp_path,
                    },
                );
            }

            Ok(AppDir {
                home_dir: uvm_home,
                log_dir: log_path,
                language_dirs,
            })
        } else {
            Err(FSError::AppDirError())
        }
    }

    pub fn get_home_dir(&self) -> &Path {
        self.home_dir.as_path()
    }

    pub fn get_log_dir(&self) -> &Path {
        self.log_dir.as_path()
    }

    pub fn get_language_dir(&self, name: &str) -> Option<&LanguageDir> {
        self.language_dirs.get(name)
    }
}

impl LanguageDir {
    pub fn get_home_dir(&self) -> &Path {
        self.home_dir.as_path()
    }

    pub fn get_versions_dir(&self) -> &Path {
        self.versions_dir.as_path()
    }

    pub fn get_current_dir(&self) -> &Path {
        self.current_dir.as_path()
    }

    pub fn get_tmp_dir(&self) -> &Path {
        self.tmp_dir.as_path()
    }
}

pub fn decompress(file_path: &Path, to_path: &Path) -> Result<Option<String>> {
    debug!("decompress from {:?} to {:?}", file_path, to_path);
    let folder = match file_path.extension() {
        Some(ext) => {
            if ext == "zip" {
                debug!("use unzip");
                unzip(file_path, to_path)?
            } else if ext == "7z" {
                debug!("use sevenz");
                sevenz(file_path, to_path)?
            } else {
                debug!("use deflate");
                deflate(file_path, to_path)?
            }
        }
        None => return Err(FSError::UnsupportedFile("")),
    };

    let result = match folder {
        Some(f) => {
            if f.ends_with('/') {
                Some(f[0..(f.len() - 1)].to_string())
            } else {
                Some(f)
            }
        }
        None => None,
    };

    Ok(result)
}

fn deflate(file_path: &Path, to_path: &Path) -> Result<Option<String>> {
    let tar_gz = fs::File::open(file_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(to_path)?;

    let tar_gz = fs::File::open(file_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let top_folder: Option<PathBuf> = match archive.entries()?.filter_map(|e| e.ok()).next() {
        Some(v) => {
            Some(v.path()?.into_owned().clone())
        },
        None => None,
    };

    match top_folder {
        Some(p) => {
            let mut components = p.components();
            let top = match components.next() {
                Some(c) => {
                    if c == Component::RootDir || c == Component::CurDir || c == Component::ParentDir {
                        components.next()
                    } else {
                        Some(c)
                    }
                },
                None => None,
            };
            Ok(top.map(|t| t.as_os_str().to_str().unwrap().to_string()))
        },
        None => Ok(None),
    }
}

fn sevenz(file_path: &Path, to_path: &Path) -> Result<Option<String>> {
    let mut top_folder: Option<PathBuf> = Option::None;
    sevenz_rust::decompress_file_with_extract_fn(file_path, to_path, |entry, reader, dest| {
        if entry.is_directory()
            && (top_folder.is_none() || top_folder.as_ref().is_some_and(|f| f.starts_with(dest)))
        {
            top_folder = Some(dest.clone());
        }
        default_entry_extract_fn(entry, reader, dest)
    })?;
    match top_folder {
        Some(folder) => match folder.file_name() {
            Some(s) => match s.to_str() {
                Some(v) => Ok(Some(v.to_string())),
                None => Ok(None),
            },
            None => Ok(None),
        },
        None => Ok(None),
    }
}

fn unzip(file_path: &Path, to_path: &Path) -> Result<Option<String>> {
    let mut top_folder = Option::None;
    let file = fs::File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut item = archive.by_index(i)?;
        let outpath = match item.enclosed_name() {
            Some(p) => to_path.join(p),
            None => continue,
        };
        {
            let comment = item.comment();
            if !comment.is_empty() {
                println!("file {i} comment: {comment}");
            }
        }
        if (*item.name()).ends_with('/') {
            if top_folder.is_none()
                || top_folder
                    .as_ref()
                    .is_some_and(|f: &String| f.len() > item.name().len())
            {
                top_folder = Some(item.name().to_string());
            }
            fs::create_dir_all(&outpath)?
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut item, &mut outfile)?;
        }
    }
    Ok(top_folder)
}

#[cfg(target_os = "windows")]
pub fn make_link(link_dir: &Path, origin_dir: &Path) -> Result<()> {
    use log::error;
    use std::fs::remove_dir;
    use std::os::windows::fs::symlink_dir;

    debug!("make link from {:?} to {:?}", link_dir, origin_dir);
    if let Err(err) = symlink_dir(origin_dir, link_dir) {
        debug!("failed to make symlink: {}, try cmd", err);
        if link_dir.exists() {
            remove_dir(link_dir)?;
        }
        let status = std::process::Command::new("cmd.exe")
            .arg("/c")
            .arg("mklink")
            .arg("/j")
            .arg(link_dir)
            .arg(origin_dir)
            .stdout(std::process::Stdio::null())
            .status()?;
        if !status.success() {
            error!("cmd mklink failed, status: {:?}", status.code());
            Err(FSError::LinkError())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn make_link(from_dir: &Path, to_dir: &Path) -> Result<()> {
    use std::os::unix::fs::symlink;
    symlink(to_dir, from_dir)?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn remove_link(dir: &Path) -> Result<()> {
    use std::fs::remove_dir;
    remove_dir(dir)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn remove_link(dir: &Path) -> Result<()> {
    use std::fs::remove_file;
    remove_file(dir)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum FSError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Cannot find user home")]
    AppDirError(),
    #[error("{0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("{0}")]
    SevenZ(#[from] sevenz_rust::Error),
    #[error("make symbol link error")]
    LinkError(),
    #[error("file type '{0}' is not supported")]
    UnsupportedFile(&'static str),
}

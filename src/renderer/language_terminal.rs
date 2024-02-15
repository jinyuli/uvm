use colored::Colorize;
use crate::executor::{LanguageExecutor, LanguageContext, ExecutorContext, InstallResult, VenvResult, UseResult};
use crate::tool::logger::error;
use super::uvm_renderer::{UvmRenderer, LanguageRenderer};
use super::terminal::TerminalRenderer;

/// LanguageTerminalRenderer executes `LanguageExecutor` and renders output on terminal.
pub struct LanguageTerminalRenderer<T> {
    lang: T,
}

impl<T> LanguageTerminalRenderer<T> {
    pub fn new(lang: T) -> Self {
        LanguageTerminalRenderer{
            lang,
        }
    }
}

impl<T> TerminalRenderer for LanguageTerminalRenderer<T> {}

impl<T> UvmRenderer for LanguageTerminalRenderer<T> {}

impl<'a, L:LanguageContext, T: LanguageExecutor<'a, L>> LanguageRenderer<'a, L> for LanguageTerminalRenderer<T> {
    fn list(&self, local_only: bool, context: &'a ExecutorContext<'a, L>) {
        match self.lang.list(local_only, context) {
            Ok(versions) => {
                for version in versions {
                    let line = if version.inuse {
                        format!("{} {}", "*".green(), version.version.green())
                    } else if version.installed {
                        format!("  {}", version.version.green())
                    } else {
                        format!("  {}", version.version)
                    };
                    self.print_line(line);
                }
            },
            Err(err) => {
                error!("failed to execute `list` command:{}", err);
                self.print_line(format!("failed to execute `list` command:\n\t{}", err.to_string().red()));
            },
        }
    }

    fn install(&self, version: String, context: &'a ExecutorContext<'a, L>) {
        match self.lang.install(version.clone(), context) {
            Ok(r) => {
                match r {
                    InstallResult::Success => {
                        self.print_line(format!("installed version {} successfully.", version.green()));
                    },
                    InstallResult::SuccessNeedHint => {
                        self.print_line(format!("installed version {} successfully.", version.green()));
                        self.print_line(self.lang.get_env_hint(context.language_dir.get_home_dir()));
                    },
                    InstallResult::VersionInstalled => {
                        self.print_line(format!("version {} is already install.", version));
                    },
                }
            },
            Err(err) => {
                error!("failed to execute `install` command:{}", err);
                self.print_line(format!("failed to execute `install` command:\n\t{}", err.to_string().red()));
            }
        }
    }

    fn uninstall(&self, version: String, context: &'a ExecutorContext<'a, L>) {
        match self.lang.uninstall(version.clone(), context) {
            Ok(_) => {
                self.print_line(format!("uninstalled version {} successfully.", version.green()));
            },
            Err(err) => {
                error!("failed to execute `uninstall` command:{}", err);
                self.print_line(format!("failed to execute `uninstall` command:\n\t{}", err.to_string().red()));
            }
        }
    }

    fn unuse(&self, context: &'a ExecutorContext<'a, L>) {
        match self.lang.unuse(context) {
            Ok(_) => {
                self.print_line(format!("unuse uvm {} successfully.", self.lang.name()));
            },
            Err(err) => {
                error!("failed to execute `unuse` command:{}", err);
                self.print_line(format!("failed to execute `unuse` command:\n\t{}", err.to_string().red()));
            }
        }
    }

    fn select(&self, version: String, context: &'a ExecutorContext<'a, L>) {
        match self.lang.select(version.clone(), context) {
            Ok(r) => {
                match r {
                    UseResult::Success => {
                        self.print_line(format!("use version {} successfully.", version.green()));
                    },
                    UseResult::SuccessNeedHint => {
                        self.print_line(format!("use version {} successfully.", version.green()));
                        self.print_line(self.lang.get_env_hint(context.language_dir.get_home_dir()));
                    },
                    UseResult::VersionAlreadyUsed => {
                        self.print_line(format!("current used version is already {}.", version.green()));
                    },
                }
            },
            Err(err) => {
                error!("failed to execute `use` command:{}", err);
                self.print_line(format!("failed to execute `use` command:\n\t{}", err.to_string().red()));
            }
        }
    }

    fn venv(&self, version: String, dir_name: String, context: &'a ExecutorContext<'a, L>) {
        match self.lang.venv(version.clone(), dir_name.clone(), context) {
            Ok(r) => {
                match r {
                    VenvResult::Success => {
                        self.print_line(format!("create virtual environment with version {} successfully.", version.green()));
                        if cfg!(target_os = "windows") {
                            self.print_line(format!("execute `{}` to activiate the virtual environment.", format!("./{}/activiate.ps1", dir_name).green()));
                            self.print_line(format!("execute `{}` to deactiviate the virtual environment.", format!("./{}/deactiviate.ps1", dir_name).green()));
                        } else {
                            self.print_line(format!("execute `{}` to activiate the virtual environment.", format!("source .\\{}\\activiate.sh", dir_name).green()));
                            self.print_line(format!("execute `{}` to deactiviate the virtual environment.", format!("source .\\{}\\deactiviate.sh", dir_name).green()));
                        }
                    },
                    VenvResult::VersionAlreadyUsed => {
                        self.print_line(format!("current virtual environment is already version {}.", version.green()));
                    },
                }
            },
            Err(err) => {
                error!("failed to execute `venv` command:{}", err);
                self.print_line(format!("failed to execute `venv` command:\n\t{}", err.to_string().red()));
            }
        }
    }
}

use crate::{
    executor::{
        load_config, ConfigContext, ConfigKey, ExecutorContext, GeneralLanguageContext,
        GolangExecutor, JavaExecutor, JavaLanguageContext, LanguageExecutor, NodeExecutor,
    },
    renderer::{
        ConfigRenderer, ConfigTerminalRenderer, LanguageRenderer, LanguageTerminalRenderer,
    },
    tool::{
        args::{
            ConfigCommand, ConfigSubCommand, GoCommand, JavaLanguageCommand, LanguageCommand,
            TopCommand,
        },
        fs::AppDir,
        logger::{debug, init_logger},
    },
};
use log::error;
use std::{
    collections::{HashMap, HashSet},
    env::consts::{ARCH, OS},
};
use tool::args::{JavaCommand, NodeCommand};

mod executor;
mod renderer;
mod tool;

fn main() {
    let app_dir = AppDir::new().unwrap();
    init_logger(app_dir.get_log_dir()).unwrap();
    let commands: tool::args::UVMCommand = argh::from_env();
    debug!("args: {:?}", commands);
    match commands.top {
        // TopCommand::Config(config_cmd) => {
        //     execute_config(config_cmd, &app_dir);
        // }
        TopCommand::Go(go_cmd) => {
            execute_golang(go_cmd, &app_dir);
        }
        TopCommand::Node(node_cmd) => {
            execute_node(node_cmd, &app_dir);
        }
        TopCommand::Java(java_cmd) => {
            execute_java(java_cmd, &app_dir);
        }
        TopCommand::Update(_) => {
            execute_update();
        }
    }
}

fn execute_update() {
    let release_update = match self_update::backends::github::Update::configure()
        .repo_owner("jinyuli")
        .repo_name("uvm")
        .bin_name("uvm")
        .show_download_progress(true)
        .current_version(self_update::cargo_crate_version!())
        .build()
    {
        Ok(u) => u,
        Err(e) => {
            error!("failed to check update: {}", e);
            return;
        }
    };
    match release_update.update() {
        Ok(s) => match s {
            self_update::Status::UpToDate(v) => {
                println!("uvm has been updated to {}.", v);
            }
            self_update::Status::Updated(v) => {
                println!("uvm {} is the latest version.", v);
            }
        },
        Err(e) => {
            error!("failed to check update: {}", e);
        }
    };
}

fn execute_config(cmd: ConfigCommand, app_dir: &AppDir) {
    let executor = executor::ConfigExecutor {};
    let terminal = ConfigTerminalRenderer::new(executor);
    let context = ConfigContext {
        home_dir: app_dir.get_home_dir().to_path_buf(),
    };
    match cmd.command {
        ConfigSubCommand::Del(del) => {
            let mut keys = HashSet::new();
            if del.proxy {
                keys.insert(ConfigKey::Proxy);
            }
            terminal.del(keys, context);
        }
        ConfigSubCommand::Get(get) => {
            let mut keys = HashSet::new();
            if get.proxy {
                keys.insert(ConfigKey::Proxy);
            }
            terminal.get(keys, context);
        }
        ConfigSubCommand::Set(set) => {
            let mut kvs = HashMap::new();
            if let Some(proxy) = set.proxy {
                kvs.insert(ConfigKey::Proxy, proxy);
            }
            terminal.set(kvs, context);
        }
    }
}

fn execute_java(cmd: JavaCommand, app_dir: &AppDir) {
    let lang = JavaExecutor::new();
    let lang_dir = app_dir
        .get_language_dir(lang.name())
        .expect("should have dir for go");
    let terminal = LanguageTerminalRenderer::new(lang);
    let config = load_config(app_dir.get_home_dir());
    let mut context = ExecutorContext::<JavaLanguageContext> {
        language_context: None,
        proxy: config.proxy,
        language_dir: lang_dir,
        filter: Option::None,
        arch: ARCH,
        os: OS,
    };
    execute_java_command(cmd.command, terminal, &mut context);
}

fn execute_golang(cmd: GoCommand, app_dir: &AppDir) {
    let lang = GolangExecutor::new();
    let lang_dir = app_dir
        .get_language_dir(lang.name())
        .expect("should have dir for go");
    let terminal = LanguageTerminalRenderer::new(lang);
    let config = load_config(app_dir.get_home_dir());
    let mut context = ExecutorContext::<GeneralLanguageContext> {
        language_context: None,
        proxy: config.proxy,
        language_dir: lang_dir,
        filter: Option::None,
        arch: ARCH,
        os: OS,
    };
    execute_command(cmd.command, terminal, &mut context);
}

fn execute_node(cmd: NodeCommand, app_dir: &AppDir) {
    let lang = NodeExecutor::new();
    let lang_dir = app_dir
        .get_language_dir(lang.name())
        .expect("should have dir for node");
    let terminal = LanguageTerminalRenderer::new(lang);
    let config = load_config(app_dir.get_home_dir());
    let mut context = ExecutorContext::<GeneralLanguageContext> {
        language_context: None,
        proxy: config.proxy,
        language_dir: lang_dir,
        filter: Option::None,
        arch: ARCH,
        os: OS,
    };
    execute_command(cmd.command, terminal, &mut context);
}

fn execute_command<'a, T: LanguageExecutor<'a, GeneralLanguageContext>>(
    cmd: LanguageCommand,
    terminal: LanguageTerminalRenderer<T>,
    context: &'a mut ExecutorContext<'a, GeneralLanguageContext>,
) {
    match cmd {
        LanguageCommand::List(list) => {
            terminal.list(list.local, context);
        }
        LanguageCommand::Install(install) => {
            context.merge(Some(GeneralLanguageContext {
                no_use: install.no_use,
            }));
            terminal.install(install.version, context);
        }
        LanguageCommand::Use(use_cmd) => {
            terminal.select(use_cmd.version, context);
        }
        LanguageCommand::Unuse(_) => {
            terminal.unuse(context);
        }
        LanguageCommand::Uninstall(uninstall) => {
            terminal.uninstall(uninstall.version, context);
        }
        LanguageCommand::VirtualEnv(venv) => {
            terminal.venv(venv.version, venv.dir, context);
        }
    }
}

fn execute_java_command<'a, T: LanguageExecutor<'a, JavaLanguageContext>>(
    cmd: JavaLanguageCommand,
    terminal: LanguageTerminalRenderer<T>,
    context: &'a mut ExecutorContext<'a, JavaLanguageContext>,
) {
    match cmd {
        JavaLanguageCommand::List(list) => {
            if let Some(v) = list.vendor {
                context.merge(Some(JavaLanguageContext {
                    vendor: v,
                    no_use: true,
                }));
            }
            terminal.list(list.local, context);
        }
        JavaLanguageCommand::Install(install) => {
            context.merge(Some(JavaLanguageContext {
                vendor: install.vendor.clone(),
                no_use: install.no_use,
            }));
            terminal.install(install.version, context);
        }
        JavaLanguageCommand::Use(use_cmd) => {
            context.merge(Some(JavaLanguageContext {
                vendor: use_cmd.vendor.clone(),
                no_use: true,
            }));
            let version = if use_cmd.version.starts_with(&use_cmd.vendor) {
                use_cmd.version.clone()
            } else {
                format!("{}-{}", use_cmd.vendor, use_cmd.version)
            };
            terminal.select(version, context);
        }
        JavaLanguageCommand::Unuse(_) => {
            terminal.unuse(context);
        }
        JavaLanguageCommand::Uninstall(uninstall) => {
            context.merge(Some(JavaLanguageContext {
                vendor: uninstall.vendor.clone(),
                no_use: true,
            }));
            let version = if uninstall.version.starts_with(&uninstall.vendor) {
                uninstall.version.clone()
            } else {
                format!("{}-{}", uninstall.vendor, uninstall.version)
            };
            terminal.uninstall(version, context);
        }
        JavaLanguageCommand::VirtualEnv(venv) => {
            context.merge(Some(JavaLanguageContext {
                vendor: venv.vendor.clone(),
                no_use: true,
            }));
            let version = if venv.version.starts_with(&venv.vendor) {
                venv.version.clone()
            } else {
                format!("{}-{}", venv.vendor, venv.version)
            };
            terminal.venv(version, venv.dir, context);
        }
    }
}

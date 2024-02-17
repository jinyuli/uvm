use argh::FromArgs;


#[derive(FromArgs, PartialEq, Debug)]
/// uvm - unified version manager, provides version management for programming languages, currently supports NodeJS, Golang and Java.
pub struct UVMCommand {
    #[argh(subcommand)]
    pub top: TopCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum TopCommand {
    // Config(ConfigCommand),
    Update(AppUpdateCommand),
    Version(AppVersionCommand),
    Go(GoCommand),
    Node(NodeCommand),
    Java(JavaCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// show uvm version.
#[argh(subcommand, name="version")]
pub struct AppVersionCommand {
}

#[derive(FromArgs, PartialEq, Debug)]
/// update uvm.
#[argh(subcommand, name="update")]
pub struct AppUpdateCommand {
}

#[derive(FromArgs, PartialEq, Debug)]
/// configuration for uvm. 
#[argh(subcommand, name="config")]
pub struct ConfigCommand {
    #[argh(subcommand)]
    pub command: ConfigSubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum ConfigSubCommand {
    Get(ConfigGetCommand),
    Set(ConfigSetCommand),
    Del(ConfigDelCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// delete a configuration option
#[argh(subcommand, name="del")]
pub struct ConfigDelCommand {
    /// proxy used during install/list command
    #[argh(switch)]
    pub proxy: bool,

    /// data dir to store downloaded package
    #[argh(switch)]
    pub data_dir: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// set value for configuration options.
#[argh(subcommand, name="set")]
pub struct ConfigSetCommand {
    /// proxy used during install/list command
    #[argh(option)]
    pub proxy: Option<String>,

    /// data dir to store downloaded/installed package
    #[argh(option)]
    pub data_dir: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// get value from configuration
#[argh(subcommand, name="get")]
pub struct ConfigGetCommand {
    /// proxy used during install/list command
    #[argh(switch)]
    pub proxy: bool,
}


#[derive(FromArgs, PartialEq, Debug)]
/// version managment commands for Golang
#[argh(subcommand, name="go")]
pub struct GoCommand {
    #[argh(subcommand)]
    pub command: LanguageCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum LanguageCommand {
    List(ListCommand),
    Install(InstallCommand),
    Uninstall(UninstallCommand),
    Use(UseCommand),
    Unuse(UnuseCommand),
    VirtualEnv(VirtualEnvCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// version managment commands for Node
#[argh(subcommand, name="node")]
pub struct NodeCommand {
    #[argh(subcommand)]
    pub command: LanguageCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
/// list remote/local versions.
#[argh(subcommand, name="list")]
pub struct ListCommand {
    /// list local only
    #[argh(switch)]
    pub local: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// install a released version.
#[argh(subcommand, name="install")]
pub struct InstallCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,

    /// install only
    #[argh(switch)]
    pub no_use: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// uninstall a version
#[argh(subcommand, name="uninstall")]
pub struct UninstallCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// switch used version
#[argh(subcommand, name="use")]
pub struct UseCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// disable current used version, to reuse, please execute command `use`
#[argh(subcommand, name="unuse")]
pub struct UnuseCommand {
}

#[derive(FromArgs, PartialEq, Debug)]
/// update a version
#[argh(subcommand, name="update")]
pub struct UpdateCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// create a virtual environment
#[argh(subcommand, name="venv")]
pub struct VirtualEnvCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,

    /// directory name, '.venv' by default 
    #[argh(option, default="default_venv_dir()")]
    pub dir: String,
}

fn default_venv_dir() -> String {
    ".venv".to_string()
}

#[derive(FromArgs, PartialEq, Debug)]
/// version managment commands for Java
#[argh(subcommand, name="java")]
pub struct JavaCommand {
    #[argh(subcommand)]
    pub command: JavaLanguageCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum JavaLanguageCommand {
    List(JavaListCommand),
    Install(JavaInstallCommand),
    Uninstall(JavaUninstallCommand),
    Use(JavaUseCommand),
    Unuse(UnuseCommand),
    VirtualEnv(JavaVirtualEnvCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// list remote/local versions
#[argh(subcommand, name="list")]
pub struct JavaListCommand {
    /// list local only
    #[argh(switch)]
    pub local: bool,

    /// for remote only, including openjdk, corretto. 
    #[argh(option)]
    pub vendor: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// install a released version
#[argh(subcommand, name="install")]
pub struct JavaInstallCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,

    /// install only
    #[argh(switch)]
    pub no_use: bool,

    /// vendors, including openjdk, corretto. 
    #[argh(option)]
    pub vendor: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// uninstall a version
#[argh(subcommand, name="uninstall")]
pub struct JavaUninstallCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,

    /// vendors, including openjdk, corretto. 
    #[argh(option)]
    pub vendor: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// switch used version
#[argh(subcommand, name="use")]
pub struct JavaUseCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,

    /// vendors, including openjdk, corretto. 
    #[argh(option)]
    pub vendor: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Update a version
#[argh(subcommand, name="update")]
pub struct JavaUpdateCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// create a virtual environment with a version, by default create a folder name .venv
#[argh(subcommand, name="venv")]
pub struct JavaVirtualEnvCommand {
    /// version to install
    #[argh(option, short='v')]
    pub version: String,

    /// directory name, '.venv' by default 
    #[argh(option, default="default_venv_dir()")]
    pub dir: String,

    /// vendors, including openjdk, corretto. 
    #[argh(option)]
    pub vendor: String,
}

# UVM

**uvm**(unified version manager) is a command-line tool for Linux, Windows and MacOS, it aims to provide convenient methods to manage multiple versions of development kit for various progamming languages.

## Install

### Automated

- Linux/macOS (bash/zsh)

  for linux, 'gnu' is the default abi in [target triple](https://doc.rust-lang.org/cargo/commands/cargo-build.html), if you are not sure what to use, use the default.
  ```shell
  $ curl -sSL https://raw.githubusercontent.com/voidint/g/master/install.sh | bash
  ```
  the script accepts one argument `--abi`.

  to enable `uvm` command in current shell
  ```shell
  $ source "$HOME/.g/env"
  ```

- Windows (pwsh)

  ```pwsh
  $ iwr https://raw.githubusercontent.com/voidint/g/master/install.ps1 -useb | iex
  ```
  the script accepts two parameters `-abi`(by default it's 'msvc') and `-arch`(by default it's 'x86_64')

### Manual 

- Linux/MacOS
    - Create a directory for `uvm` (recommended: `~/.uvm`), download executable file from [releases](https://github.com/jinyuli/uvm/releases), and copy it to the `bin` subdirectory of the `uvm` directory (i.e. `~/.uvm/bin`), rename it to `uvm`.
    - Update system `PATH` to include `uvm` path.

- Windows
    - Create a directory for `uvm` (recommended: `~/.uvm`), download executable file from [releases](https://github.com/jinyuli/uvm/releases), and copy it to the `bin` subdirectory of the `uvm` directory (i.e. `~/.uvm/bin`), rename it to `uvm.exe`.
    - Update system `PATH` to include `uvm.exe` path.

## Features

### Golang
* **list** all released versions.
* **list** local installed versions.
* **install** a released version.
* **use** an installed version.
* create a virtual environment(**venv**) in a workspace.
* **uninstall** an installed version.

### Node
* **list** all released versions.
* **list** local installed versions.
* **install** a released version.
* **use** an installed version.
* create a virtual environment(**venv**) in a workspace.
* **uninstall** an installed version.

### Java
* support [OpenJDK](https://jdk.java.net/) and [Amazon Corretto](https://docs.aws.amazon.com/corretto/).
* **list** all released versions.
* **list** local installed versions.
* **install** a released version.
* **use** an installed version.
* create a virtual environment(**venv**) in a workspace.
* **uninstall** an installed version.

## Usage

update **uvm** to latest version:
```shell
$ uvm update
```

list all released versions of Golang:
```shell
$ uvm go list
```

list installed versions of Golang:
```shell
$ uvm go list --local
  1.20.1
* 1.20.10
  1.21.6
```

install Golang 1.21.6:
```shell
$ uvm go install -v 1.21.6
```

switch default global Golang version to 1.21.6:
```shell
$ uvm go use -v 1.21.6
```

switch default global Golang version to 1.21.6:
```shell
$ uvm go use -v 1.21.6
```

create a virtual environment with Golang 1.20.1 in current folder:
```shell
$ uvm go venv -v 1.20.1
```
by default this will create a folder named `.venv`

activiate virtual environment:
```shell
$ source ./.venv/activiate.sh
(go)$
```

deactiviate virtual environment:
```shell
$ source ./.venv/deactiviate.sh
$
```

**Node** has similar commands.

**Java** is a little different, each command must specify vendor with `--vendor`, currently supported vendors includs *openjdk*, *corretto*.

for example, install Java 20 from Amazon Corretto:
```shell
$ uvm java install -v 20 --vendor corretto
```


## TODO

- [ ] configuration for uvm
- [ ] support mirrors
- [ ] support more languages
- [ ] show progress while downloading files
- [ ] support more shells for venv

## Acknowledgement

Thanks to tools like [nvm](https://github.com/nvm-sh/nvm), [n](https://github.com/tj/n), [rvm](https://github.com/rvm/rvm), [g](https://github.com/voidint/g), [using-virtual-environments](https://github.com/JeNeSuisPasDave/using-virtual-environments) for providing valuable ideas.

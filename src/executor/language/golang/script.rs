use std::fs::{remove_file, File};
use std::io::{Error, Write};
use std::path::Path;

pub type Result<T> = std::result::Result<T, Error>;

pub fn generate_scripts(dir: &Path) -> Result<()> {
    for (script, file_name) in get_script_infos() {
        let file_path = dir.join(file_name);
        if file_path.exists() && file_path.is_file() {
            remove_file(&file_path)?;
        }
        let mut file = File::create(&file_path)?;
        file.write_all(script.as_bytes())?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn get_script_infos<'a>() -> [(&'a str, &'a str); 2] {
    [
        (PS1_ACTIVIATE, "activiate.ps1"),
        (PS1_DEACTIVIATE, "deactiviate.ps1"),
    ]
}

#[cfg(not(target_os = "windows"))]
fn get_script_infos<'a>() -> [(&'a str, &'a str); 2] {
    [
        (SHELL_ACTIVIATE, "activiate.sh"),
        (SHELL_DEACTIVIATE, "deactiviate.sh"),
    ]
}

#[cfg(target_os = "windows")]
const PS1_ACTIVIATE: &str = r###"
$scriptDir = $PSScriptRoot

# Convert input paths to absolute paths
#
$LocalGolangDirectory = "$scriptDir\go"
$LocalGoPathDirectory = "$scriptDir\go_path"

# Check that Go exists
#
$golangBinPath = "$LocalGolangDirectory\bin"
If (-not (Test-Path "$golangBinPath\go.exe")) {
    Write-Host ""
    Write-Host "ERROR: Go is not installed in '.\$LocalGolangDirectory'."
    Write-Host "Cannot activate Go virtual environment."
    Write-Host ""
    return
}

if (-not (Test-Path "$LocalGoPathDirectory")) {
    New-Item -Path "$scriptDir" -Name "go_path" -ItemType "directory" | out-null
}

# Set GOROOT, GOPATH
#
if ($Env:GOROOT -ne "$LocalGolangDirectory") {
    $Env:GOROOT_PRE_ = $Env:GOROOT
}
if ($Env:GOPATH -ne "$LocalGoPathDirectory") {
    $Env:GOPATH_PRE_ = $Env:GOPATH
}
$Env:GOROOT = "$LocalGolangDirectory"
$Env:GOPATH = "$LocalGoPathDirectory"
[System.Collections.ArrayList]$pathParts = $Env:Path.split(';')
$found = $false
If ($pathParts.Contains("$Env:GOROOT\bin")) {
    $pathParts.Remove("$Env:GOROOT\bin")
    $found = $true
}
if ($found) {
    $Env:Path = $pathParts -Join ';'
}
$Env:Path = "$Env:GOROOT\bin" + ";" +  "$Env:Path"

# Adjust the prompt
#
# Note: borrowed this idea from the Python 2.7 activate.ps1 script
#
If (Test-Path function:_old_virtual_prompt_go) {
    # Restore old prompt if it existed
    #
    $Function:prompt = $Function:_old_virtual_prompt_go
    Remove-Item function:\_old_virtual_prompt_go
}
Function global:_old_virtual_prompt_go { "" }
$Function:_old_virtual_prompt_go = $Function:prompt
Function global:prompt {
    # Add a prefix to the current prompt, but don't discard it.
    Write-Host "(go) " -nonewline
    & $Function:_old_virtual_prompt_go
}
"###;

#[cfg(target_os = "windows")]
const PS1_DEACTIVIATE: &str = r###"
If (-Not (Test-Path env:\GOROOT) -or -Not (Test-Path $Env:GOROOT)) {
    Write-Host "Go virtual environment not active."
    Write-Host ""
    return
}

# Remove Go binary directories from the $Path
#
[System.Collections.ArrayList]$pathParts = $Env:Path.split(';')
$pathParts.Remove("$Env:GOROOT\bin")
$Env:Path = $pathParts -Join ';'
$Env:GOROOT = $Env:GOROOT_PRE_
$Env:GOPATH = $Env:GOPATH_PRE_
$Env:GOROOT_PRE_ = ""
$Env:GOPATH_PRE_ = ""

# Remove the virtual enviroment indication from the command prompt
#
If (Test-Path function:_old_virtual_prompt_go) {
    $function:prompt = $function:_old_virtual_prompt_go
    Remove-Item function:\_old_virtual_prompt_go
}
"###;

#[cfg(not(target_os = "windows"))]
const SHELL_ACTIVIATE: &str = r###"
#!/bin/bash
function activiate() {
    local GOLANG_ROOT_="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local GOLANG_PATH_="$(pwd)/go_path"

    # Make sure Go has a local deployment
    #
    if [[ ! -d "${GOLANG_ROOT_}/go" ]]; then
        echo "ERROR: Go is not installed at '${GOLANG_ROOT_}'." > /dev/stderr
        return 2
    fi
    if [[ ! -d "${GOLANG_ROOT_}/go/bin" ]]; then
        echo "ERROR: Go is not installed at '${GOLANG_ROOT_}'." > /dev/stderr
        return 2
    fi
    if [[ ! -f "${GOLANG_ROOT_}/go/bin/go" ]]; then
        echo "ERROR: Go is not installed at '${GOLANG_ROOT_}'." > /dev/stderr
        return 2
    fi

    # Check whether the local Go is already active
    #
    local NEW_PATH_=""
    local IFS_SAVE_="${IFS}"
    IFS=':'
    local PATH_PARTS_=($PATH)
    IFS="${IFS_SAVE_}"

    local FOUND_=0
    for PART_ in "${PATH_PARTS_[@]}"
    do
        if [[ "${GOLANG_ROOT_}/go/bin" == "${PART_}" ]]; then
            FOUND_=1
            break
        fi
    done

    # If Go is not active, set the Go environment variables, add GOROOT
    # to the path and prefix the command prompt with '(go)'
    #
    if (( 0 == $FOUND_ )); then
        # Set GOROOT, GOPATH
        local GOLANG_BASE_="${GOLANG_ROOT_}"

        if [[ "$GOROOT" != "${GOLANG_ROOT_}/go" ]]; then
            export GO_PREVGOROOT_=$GOROOT
        fi
        export GOROOT="${GOLANG_ROOT_}/go"
        local RMDIR_GOPATH_=1
        if [[ ! -d "${GOLANG_PATH_}" ]]; then
            mkdir -p "${GOLANG_PATH_}" > /dev/null
            RMDIR_GOPATH_=0
        fi

        if [[ "$GOPATH" != "${GOLANG_PATH_}" ]]; then
            export GO_PREVGOPATH_=$GOPATH
        fi
        export GOPATH="${GOLANG_PATH_}"
        # clean up intermediate directories
        # (necessary to allow subsequent git clone)
        if (( 0 == $RMDIR_GOPATH_ )); then
            rm -r "${GOPATH}"
        fi

        # Update PATH and command prompt
        #
        export PATH="${GOBIN}:${GOROOT}/bin:${PATH}"
        export PS1="(go)${PS1}"
    fi
}

if [ "$0" = "$BASH_SOURCE" ]; then
    echo "Error: Script must be sourced"
    exit 1
fi
activiate

"###;

#[cfg(not(target_os = "windows"))]
const SHELL_DEACTIVIATE: &str = r###"
#!/bin/bash
function deactiviate() {
    local DIR_ROOT_="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local GOLANG_ROOT_="${DIR_ROOT}/go/bin"

    # Check whether the local Go is already active, and build
    # a new PATH without Go while checking.
    #
    local NEW_PATH_=""
    local IFS_SAVE_="${IFS}"
    IFS=':'
    local PATH_PARTS_=($PATH)
    IFS="${IFS_SAVE_}"

    local FOUND_=0
    for PART_ in "${PATH_PARTS_[@]}"
    do
        if [[ "${GOLANG_ROOT_}" == "${PART_}" ]]; then
            FOUND_=1
            continue
        fi
        if [[ -z "${NEW_PATH_}" ]]; then
            NEW_PATH_="${PART_}"
        else
            NEW_PATH_="${NEW_PATH_}:${PART_}"
        fi
    done

    # Build a new command prompt without a '(go)' prefix
    #
    local RE_='s/^(.*)\(go\)(.*)$/\1\2/'
    local NEW_PS1_=`echo "${PS1}" | sed -E ${RE_}`

    # Update the new PATH and command prompt, if necessary
    #
    if (( 1 == $FOUND_ )); then
        export PATH="${NEW_PATH_}"
        export PS1="${NEW_PS1_}"
    fi

    # Unset the GO variables
    #
    if [[ -n "$GO_PREVGOROOT_" ]]; then
        export GOROOT=$GO_PREVGOROOT_
        unset GO_PREVGOROOT_
    elif [[ -n "$GOROOT" ]]; then
        unset GOROOT
        unset GO_PREVGOROOT_
    fi
    if [[ -n "$GO_PREVGOPATH_" ]]; then
        export GOPATH=$GO_PREVGOPATH_
        unset GO_PREVGOPATH_
    elif [[ -n "$GOPATH" ]]; then
        unset GOPATH
        unset GO_PREVGOPATH_
    fi
}

if [ "$0" = "$BASH_SOURCE" ]; then
    echo "Error: Script must be sourced"
    exit 1
fi
deactiviate

"###;

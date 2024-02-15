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

# Check that Node.js exists
#
$LocalNodeJsDirectory = "$scriptDir\node"
$jslclDir = $LocalNodeJsDirectory
$nodejsBinPath = "$LocalNodeJsDirectory"
If (-not (Test-Path "$nodejsBinPath\node.exe")) {
    Write-Host ""
    Write-Host "ERROR: Node.js is not installed in '.\$jslclDir'."
    Write-Host "Cannot activate Node.js virtual environment."
    Write-Host ""
    return
}

# Add Node.js to the path
#
[System.Collections.ArrayList]$pathParts = $Env:Path.split(';')
$found = $false
If ($pathParts.Contains("$nodejsBinPath")) {
    $pathParts.Remove($nodejsBinPath)
    $found = $true
}
if ($found) {
    $Env:Path = $pathParts -Join ';'
}
$Env:Path = "$nodejsBinPath" + ";" +  "$Env:Path"

# Set the NODE_BIN_ environment variable
#
$Env:NODE_BIN_ = "$nodejsBinPath"

# Adjust the prompt
#
# Note: borrowed this idea from the Python 2.7 activate.ps1 script
#
If (Test-Path function:_old_virtual_prompt_js) {
    # Restore old prompt if it existed
    #
    $Function:prompt = $Function:_old_virtual_prompt_js
    Remove-Item function:\_old_virtual_prompt_js
}
Function global:_old_virtual_prompt_js { "" }
$Function:_old_virtual_prompt_js = $Function:prompt
Function global:prompt {
    # Add a prefix to the current prompt, but don't discard it.
    Write-Host "(node) " -nonewline
    & $Function:_old_virtual_prompt_js
}
"###;

#[cfg(target_os = "windows")]
const PS1_DEACTIVIATE: &str = r###"
If (-Not (Test-Path env:\NODE_BIN_) -or -Not (Test-Path $Env:NODE_BIN_)) {
    Write-Host "Node.js virtual environment not active."
    Write-Host ""
    return
}

# Remove Node.js binary directories from the $Path
#
[System.Collections.ArrayList]$pathParts = $Env:Path.split(';')
$pathParts.Remove($Env:NODE_BIN_)
$Env:Path = $pathParts -Join ';'
$Env:NODE_BIN_ = ""

# Remove the virtual enviroment indication from the command prompt
#
If (Test-Path function:_old_virtual_prompt_js) {
    $function:prompt = $function:_old_virtual_prompt_js
    Remove-Item function:\_old_virtual_prompt_js
}
"###;

#[cfg(not(target_os = "windows"))]
const SHELL_ACTIVIATE: &str = r###"
#!/bin/bash
function activiate() {
    local DIR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local NODE_ROOT_="${DIR_ROOT}/node"

    # Make sure node has a local deployment
    #
    if [[ ! -f "${NODE_ROOT_}/node" ]]; then
        echo "ERROR: NodeJS is not installed at '${NODE_ROOT_}'." > /dev/stderr
        return 2
    fi

    # Check whether the local Node is already active
    #
    local NEW_PATH_=""
    local IFS_SAVE_="${IFS}"
    IFS=':'
    local PATH_PARTS_=($PATH)
    IFS="${IFS_SAVE_}"

    local FOUND_=0
    for PART_ in "${PATH_PARTS_[@]}"
    do
        if [[ "${NODE_ROOT_}" == "${PART_}" ]]; then
            FOUND_=1
            break
        fi
    done

    # If Node is not active, add prefix the command prompt with '(node)'
    #
    if (( 0 == $FOUND_ )); then
        # Update PATH and command prompt
        #
        export PATH="${NODE_ROOT_}:${PATH}"
        export PS1="(node)${PS1}"
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
    local DIR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local NODE_ROOT_="${DIR_ROOT}/node"

    # Check whether the local Node is already active, and build
    # a new PATH without java while checking.
    #
    local NEW_PATH_=""
    local IFS_SAVE_="${IFS}"
    IFS=':'
    local PATH_PARTS_=($PATH)
    IFS="${IFS_SAVE_}"

    local FOUND_=0
    for PART_ in "${PATH_PARTS_[@]}"
    do
        if [[ "${NODE_ROOT_}" == "${PART_}" ]]; then
            FOUND_=1
            continue
        fi
        if [[ -z "${NEW_PATH_}" ]]; then
            NEW_PATH_="${PART_}"
        else
            NEW_PATH_="${NEW_PATH_}:${PART_}"
        fi
    done

    # Build a new command prompt without a '(node)' prefix
    #
    local RE_='s/^(.*)\(node\)(.*)$/\1\2/'
    local NEW_PS1_=`echo "${PS1}" | sed -E ${RE_}`

    # Update the new PATH and command prompt, if necessary
    #
    if (( 1 == $FOUND_ )); then
        export PATH="${NEW_PATH_}"
        export PS1="${NEW_PS1_}"
    fi
}

if [ "$0" = "$BASH_SOURCE" ]; then
    echo "Error: Script must be sourced"
    exit 1
fi
deactiviate


"###;

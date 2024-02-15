use std::path::Path;
use std::fs::{File, remove_file};
use std::io::{Write, Error};

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

#[cfg(target_os="windows")]
fn get_script_infos<'a>() -> [(&'a str, &'a str); 2] {
    [(PS1_ACTIVIATE, "activiate.ps1"), (PS1_DEACTIVIATE, "deactiviate.ps1")]
}

#[cfg(not(target_os="windows"))]
fn get_script_infos<'a>() -> [(&'a str, &'a str); 2] {
    [(SHELL_ACTIVIATE, "activiate.sh"), (SHELL_DEACTIVIATE, "deactiviate.sh")]
}

#[cfg(target_os="windows")]
const PS1_ACTIVIATE: &str = r###"
$scriptDir = $PSScriptRoot

# Check that Java exists
#
$LocalJavaDirectory = "$scriptDir\java"
$javaBinPath = "$LocalJavaDirectory\bin"
If (-not (Test-Path "$javaBinPath\java.exe")) {
    Write-Host ""
    Write-Host "ERROR: Java is not installed in '.\$LocalJavaDirectory'."
    Write-Host "Cannot activate Java virtual environment."
    Write-Host ""
    return
}

if ($Env:JAVA_HOME -ne "$LocalJavaDirectory") {
    $Env:JAVA_HOME_PRE_ = $Env:JAVA_HOME
}
$Env:JAVA_HOME = "$LocalJavaDirectory"
# Add Java to the path
#
[System.Collections.ArrayList]$pathParts = $Env:Path.split(';')
$found = $false
If ($pathParts.Contains("$javaBinPath")) {
    $pathParts.Remove($javaBinPath)
    $found = $true
}
if ($found) {
    $Env:Path = $pathParts -Join ';'
}
$Env:Path = "$javaBinPath" + ";" +  "$Env:Path"

# Set the JAVA_BIN_ environment variable
#
$Env:JAVA_BIN_ = "$javaBinPath"

# Adjust the prompt
#
# Note: borrowed this idea from the Python 2.7 activate.ps1 script
#
If (Test-Path function:_old_virtual_prompt_java) {
    # Restore old prompt if it existed
    #
    $Function:prompt = $Function:_old_virtual_prompt_java
    Remove-Item function:\_old_virtual_prompt_java
}
Function global:_old_virtual_prompt_java { "" }
$Function:_old_virtual_prompt_java = $Function:prompt
Function global:prompt {
    # Add a prefix to the current prompt, but don't discard it.
    Write-Host "(Java) " -nonewline
    & $Function:_old_virtual_prompt_java
}
"###;

#[cfg(target_os="windows")]
const PS1_DEACTIVIATE: &str = r###"
If (-Not (Test-Path env:\JAVA_BIN_) -or -Not (Test-Path $Env:JAVA_BIN_)) {
    Write-Host "Java virtual environment not active."
    Write-Host ""
    return
}

# Remove Java bin directories from the $Path
#
[System.Collections.ArrayList]$pathParts = $Env:Path.split(';')
$pathParts.Remove($Env:JAVA_BIN_)
$Env:Path = $pathParts -Join ';'
$Env:JAVA_BIN_ = ""
$Env:JAVA_HOME = $Env:JAVA_HOME_PRE_
$Env:JAVA_HOME_PRE_ = ""

# Remove the virtual enviroment indication from the command prompt
#
If (Test-Path function:_old_virtual_prompt_java) {
    $function:prompt = $function:_old_virtual_prompt_java
    Remove-Item function:\_old_virtual_prompt_java
}
"###;

#[cfg(not(target_os="windows"))]
const SHELL_ACTIVIATE: &str = r###"
#!/bin/bash
function activiate() {
    local DIR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local JAVA_ROOT_="${DIR_ROOT}/java"

    # Make sure java has a local deployment
    #
    if [[ ! -f "${JAVA_ROOT_}/bin/java" ]]; then
        echo "ERROR: Java is not installed at '${JAVA_ROOT_}'." > /dev/stderr
        return 2
    fi

    # Check whether the local Java is already active
    #
    local NEW_PATH_=""
    local IFS_SAVE_="${IFS}"
    IFS=':'
    local PATH_PARTS_=($PATH)
    IFS="${IFS_SAVE_}"

    local FOUND_=0
    for PART_ in "${PATH_PARTS_[@]}"
    do
        if [[ "${JAVA_ROOT_}/bin" == "${PART_}" ]]; then
            FOUND_=1
            break
        fi
    done

    # If Java is not active, add JAVA_HOME and
    # prefix the command prompt with '(java)'
    #
    if (( 0 == $FOUND_ )); then
        if [[ "$JAVA_HOME" != "${JAVA_ROOT_}" ]]; then
            export JAVA_HOME_PRE_=$JAVA_HOME
        fi
        export JAVA_HOME="${JAVA_ROOT_}"

        # Update PATH and command prompt
        #
        export PATH="${JAVA_ROOT_}/bin:${PATH}"
        export PS1="(java)${PS1}"
    fi
}

if [ "$0" = "$BASH_SOURCE" ]; then
    echo "Error: Script must be sourced"
    exit 1
fi
activiate

"###;

#[cfg(not(target_os="windows"))]
const SHELL_DEACTIVIATE: &str = r###"
#!/bin/bash
function deactiviate() {
    local DIR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local JAVA_ROOT_="${DIR_ROOT}/java"

    # Check whether the local java is already active, and build
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
        if [[ "${JAVA_ROOT_}/bin" == "${PART_}" ]]; then
            FOUND_=1
            continue
        fi
        if [[ -z "${NEW_PATH_}" ]]; then
            NEW_PATH_="${PART_}"
        else
            NEW_PATH_="${NEW_PATH_}:${PART_}"
        fi
    done

    # Build a new command prompt without a '(java)' prefix
    #
    local RE_='s/^(.*)\(java\)(.*)$/\1\2/'
    local NEW_PS1_=`echo "${PS1}" | sed -E ${RE_}`

    # Update the new PATH and command prompt, if necessary
    #
    if (( 1 == $FOUND_ )); then
        export PATH="${NEW_PATH_}"
        export PS1="${NEW_PS1_}"
    fi

    # Unset the Java variables
    #
    if [[ -n "$JAVA_HOME_PRE_" ]]; then
        export JAVA_HOME=$JAVA_HOME_PRE_
        unset JAVA_HOME_PRE_
    elif [[ -n "$JAVA_HOME" ]]; then
        unset JAVA_HOME
        unset JAVA_HOME_PRE_
    fi
}

if [ "$0" = "$BASH_SOURCE" ]; then
    echo "Error: Script must be sourced"
    exit 1
fi
deactiviate

"###;


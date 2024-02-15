$release = "0.1.0"
$os = "windows"
$arch = "amd64"

$base_dir = "$HOME\.uvm"
$dest_file = "${base_dir}\downloads\uvm${release}.${os}-${arch}.zip"
$url = "https://github.com/jinyuli/uvm/releases/download/v${release}/uvm${release}.${os}-${arch}.zip"

function NewDirs () {
    New-Item -Force -Path "$base_dir\downloads", "$base_dir\bin" -ItemType "directory"
}

function CleanDirs() {
    Remove-Item -Recurse -Path "$base_dir"
}

function DownloadRelease() {
    Invoke-WebRequest -Uri "$url" -OutFile "$dest_file"
}

function Install () {
    Expand-Archive -Path "$dest_file" -DestinationPath "$base_dir\bin\" -Force
}

function setPath() {
    $paths = [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User) -split ';'
    $newPaths = @("$base_dir\bin")

    foreach ($p in $newPaths) {
        if ($p -in $paths) {
            Write-Output "$p already exists"
            continue
        }

        [System.Environment]::SetEnvironmentVariable(
            "PATH",
            [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User) + "$p;",
            [System.EnvironmentVariableTarget]::User
        )
        Write-Host -ForegroundColor Green "$p appended"
    }
}

Write-Host -ForegroundColor Blue "[1/3] Downloading ${url}"
NewDirs
DownloadRelease

Write-Host -ForegroundColor Blue "[2/3] Install uvm to the ${base_dir}\bin"
Install

Write-Host -ForegroundColor Blue "[3/3] Set environment variables"
setPath

Write-Host -ForegroundColor Green "uvm $release installed!"

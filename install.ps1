param(
    $abi = "msvc",
    $arch = "x86_64"
)

$release = "0.1.0-alpha.10"
$os = "windows"

$base_dir = "$HOME\.uvm"
$dest_file = "${base_dir}\downloads\uvm${release}.${os}-${arch}.exe"
$url = "https://github.com/jinyuli/uvm/releases/download/v${release}/uvm-${arch}-pc-${os}-${abi}.exe"

function Uvm-New-Dirs {
    New-Item -Force -Path "$base_dir\downloads", "$base_dir\bin" -ItemType "directory"
}

function Uvm-Clean-Dirs {
    Remove-Item -Recurse -Path "$base_dir\downloads"
}

function Uvm-Download-Release {
    $StatusCode = 400
    try {
        Invoke-WebRequest -Uri "$url" -OutFile "$dest_file"
        $StatusCode = 200
    } catch {
        Write-Error "Failed to download file: $PSItem"
    }
    return $StatusCode
}

function Uvm-Install {
    Copy-Item "$dest_file" -Destination "$base_dir\bin\uvm.exe"
}

function Uvm-Set-Path {
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
Uvm-New-Dirs

$StatusCode = Uvm-Download-Release

if ($StatusCode -eq 200) {
    Write-Host -ForegroundColor Blue "[2/3] Install uvm to the ${base_dir}\bin"
    Uvm-Install

    Write-Host -ForegroundColor Blue "[3/3] Set environment variables"
    Uvm-Set-Path

    Write-Host -ForegroundColor Green "uvm $release installed!"
}

Uvm-Clean-Dirs
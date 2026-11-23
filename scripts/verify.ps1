param(
    [switch]$Release
)

$ErrorActionPreference = 'Stop'

$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    $cargoPath = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (-not (Test-Path $cargoPath)) {
        throw 'Rust stable MSVC is required. Install it from https://rustup.rs/ and open a new shell.'
    }
    $cargoPath = (Resolve-Path $cargoPath).Path
} else {
    $cargoPath = $cargo.Source
}

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path $vswhere)) {
    throw 'Visual Studio Build Tools with the C++ x64 workload is required.'
}
$vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vsPath) {
    throw 'No Visual Studio C++ x64 Build Tools installation was found.'
}
$vcvars = Join-Path $vsPath 'VC\Auxiliary\Build\vcvars64.bat'
if (-not (Test-Path $vcvars)) {
    throw "vcvars64.bat was not found at $vcvars"
}

function Invoke-CargoInMsvcEnvironment([string]$Arguments) {
    $command = 'call "{0}" >nul && "{1}" {2}' -f $vcvars, $cargoPath, $Arguments
    cmd.exe /d /c $command
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $Arguments failed with exit code $LASTEXITCODE"
    }
}

Invoke-CargoInMsvcEnvironment 'fmt --all -- --check'
Invoke-CargoInMsvcEnvironment 'clippy --all-targets -- -D warnings'
Invoke-CargoInMsvcEnvironment 'test'
if ($Release) {
    Invoke-CargoInMsvcEnvironment 'build --release'
}

Write-Host 'UmaPilot verification passed.' -ForegroundColor Green

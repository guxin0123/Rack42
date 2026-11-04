param(
    [string]$Mirror = 'https://gh-proxy.com',
    [switch]$DebugBuild,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$ExtraArgs
)

$ErrorActionPreference = 'Stop'

if (-not $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR) {
    $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR = $Mirror
}

Write-Host "Tauri bundler mirror: $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR"

$tauriArgs = @('tauri', 'build')
if ($DebugBuild) {
    $tauriArgs += '--debug'
}
if ($ExtraArgs) {
    $tauriArgs += $ExtraArgs
}

& npx.cmd @tauriArgs
exit $LASTEXITCODE

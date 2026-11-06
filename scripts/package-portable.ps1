param(
    [switch]$Debug
)

$ErrorActionPreference = 'Stop'

$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$configPath = Join-Path $root 'src-tauri\tauri.conf.json'
$config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
$version = [string]$config.version

$profile = if ($Debug) { 'debug' } else { 'release' }
$targetDir = Join-Path $root "src-tauri\target\$profile"
$exeCandidates = @(
    (Join-Path $targetDir 'rack42.exe'),
    (Join-Path $targetDir 'Rack42.exe')
)

$outputDir = Join-Path $root 'release'
$suffix = if ($Debug) { '-debug' } else { '' }
$zipPath = Join-Path $outputDir "Rack42-$version-windows-x64$suffix.zip"
$stageDir = Join-Path $outputDir 'stage'

Write-Host "==> 构建 Rack42 $version（$profile，不生成安装包）"
$tauriArgs = @('tauri', 'build', '--no-bundle')
if ($Debug) {
    $tauriArgs += '--debug'
}
& npx.cmd @tauriArgs
if ($LASTEXITCODE -ne 0) {
    throw "tauri build 失败（退出码 $LASTEXITCODE）"
}

$exePath = $exeCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $exePath) {
    throw "未找到可执行文件，请检查构建输出：$targetDir"
}

if (-not (Test-Path -LiteralPath $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

if (Test-Path -LiteralPath $stageDir) {
    $resolvedStage = (Resolve-Path -LiteralPath $stageDir).Path
    if (-not $resolvedStage.StartsWith($outputDir, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "拒绝删除非 release 目录：$resolvedStage"
    }
    Remove-Item -LiteralPath $resolvedStage -Recurse -Force
}
New-Item -ItemType Directory -Path $stageDir | Out-Null

Copy-Item -LiteralPath $exePath -Destination (Join-Path $stageDir 'Rack42.exe')
Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination $stageDir
$assets = Join-Path $root 'assets'
if (Test-Path -LiteralPath $assets) {
    Copy-Item -LiteralPath $assets -Destination $stageDir -Recurse
}
$license = Join-Path $root 'LICENSE'
if (Test-Path -LiteralPath $license) {
    Copy-Item -LiteralPath $license -Destination $stageDir
}

$guide = @"
Rack42 便携版

1. 双击 Rack42.exe 运行；需要 Windows 10 1903+ 和 WebView2 运行时。
2. 首次启动会询问是否使用便携模式。选择“是”后，数据保存在本目录的 data 文件夹，可随程序一起拷贝。
3. 本分发包为免安装压缩包，不包含安装程序。

更多说明见 README.md。
"@
Set-Content -LiteralPath (Join-Path $stageDir '使用说明.txt') -Value $guide -Encoding UTF8

if (Test-Path -LiteralPath $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}
Compress-Archive -Path (Join-Path $stageDir '*') -DestinationPath $zipPath -CompressionLevel Optimal
Remove-Item -LiteralPath $stageDir -Recurse -Force

$sizeMb = [math]::Round((Get-Item -LiteralPath $zipPath).Length / 1MB, 2)
Write-Host "==> 已生成 $zipPath（$sizeMb MB）"

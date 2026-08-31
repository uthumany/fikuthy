[CmdletBinding()]
param(
  [string]$Version = $env:UTHARNESS_VERSION,
  [string]$InstallDir = $env:UTHARNESS_INSTALL_DIR
)

$ErrorActionPreference = 'Stop'
$Repository = if ($env:UTHARNESS_REPOSITORY) { $env:UTHARNESS_REPOSITORY } else { 'uthumany/utharnessly' }
$ReleaseBaseUrl = if ($env:UTHARNESS_RELEASE_BASE_URL) { $env:UTHARNESS_RELEASE_BASE_URL.TrimEnd('/') } else { "https://github.com/$Repository/releases" }
if (-not $Version) { $Version = 'latest' }
if (-not $InstallDir) { $InstallDir = Join-Path $HOME '.local\bin' }
$node = Get-Command node -ErrorAction SilentlyContinue
if (-not $node) { throw 'Node.js 18 or newer is required for the full-screen terminal UI. Install Node.js LTS and rerun this installer.' }
$nodeMajor = [int]((& node -p "Number(process.versions.node.split('.')[0])").Trim())
if ($nodeMajor -lt 18) { throw "Node.js 18 or newer is required; found $(& node --version)" }

if ($Version -eq 'latest') {
  $ReleaseUrl = "$ReleaseBaseUrl/latest/download"
  $VersionLabel = 'latest'
} else {
  $Version = $Version.TrimStart('v')
  $ReleaseUrl = "$ReleaseBaseUrl/download/v$Version"
  $VersionLabel = "v$Version"
}

$asset = 'utharnessly-windows-x64.zip'
$temp = Join-Path ([System.IO.Path]::GetTempPath()) ("utharnessly-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $temp | Out-Null
try {
  $archive = Join-Path $temp $asset
  $checksums = Join-Path $temp 'SHA256SUMS'
  Write-Host "Downloading utharnessly $VersionLabel (windows/x64)…"
  try {
    Invoke-WebRequest "$ReleaseUrl/$asset" -OutFile $archive
  } catch {
    throw "No matching Windows release archive was found. Build from source with Git, Rust, Node 22, and pnpm: git clone https://github.com/$Repository.git; cd utharnessly; cargo build --release; pnpm --dir ui install; pnpm --dir ui build"
  }
  try {
    Invoke-WebRequest "$ReleaseUrl/SHA256SUMS" -OutFile $checksums
    $line = Get-Content $checksums | Where-Object { $_ -match [regex]::Escape($asset) } | Select-Object -First 1
    if ($line) {
      $expected = ($line -split '\s+')[0]
      $actual = (Get-FileHash $archive -Algorithm SHA256).Hash.ToLowerInvariant()
      if ($expected.ToLowerInvariant() -ne $actual) { throw 'checksum verification failed' }
    }
  } catch { throw "Checksum download or validation failed; refusing to install an unverified archive: $($_.Exception.Message)" }
  Expand-Archive -Path $archive -DestinationPath $temp -Force
  $package = Get-ChildItem $temp -Directory | Where-Object { $_.Name -like 'utharnessly-*' } | Select-Object -First 1
  if (-not $package) { throw 'release archive has no utharnessly directory' }
  if (-not (Test-Path (Join-Path $package.FullName 'utharness.exe'))) { throw 'release archive has no utharness.exe' }
  if (-not (Test-Path (Join-Path $package.FullName 'ui\dist\index.js'))) { throw 'release archive has no built terminal UI bundle' }
  New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
  Copy-Item (Join-Path $package.FullName 'utharness.exe') (Join-Path $InstallDir 'utharness.exe') -Force
  New-Item -ItemType Directory -Path (Join-Path $InstallDir 'utharnessly-ui') -Force | Out-Null
  Copy-Item (Join-Path $package.FullName 'ui\*') (Join-Path $InstallDir 'utharnessly-ui') -Recurse -Force
  $installedVersion = & (Join-Path $InstallDir 'utharness.exe') --version
  if (-not ($installedVersion -like 'utharness *')) { throw 'installed binary failed its version health check' }
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  if (($userPath -split ';') -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable('Path', (($userPath, $InstallDir) -join ';'), 'User')
  }
  Write-Host "Installed and verified $installedVersion with Node $(& node --version) in $InstallDir. Open a new terminal, then run: utharness"
} finally {
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}

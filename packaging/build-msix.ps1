# Builds the Microsoft Store package: packaging\out\Keyflex_<version>_x64.msix
#
# The package is not signed. That is correct for a Store submission: the Store
# signs it during certification. (It cannot be installed directly as it is.)
#
#   pwsh packaging/build-msix.ps1 `
#     -IdentityName "12345Publisher.Keyflex" `
#     -Publisher "CN=XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX" `
#     -PublisherDisplayName "Your Publisher Name"
#
# See packaging/README.md for where those three values come from.
param(
  [string]$IdentityName = "Keyflex.Dev",
  [string]$Publisher = "CN=Keyflex Dev",
  [string]$PublisherDisplayName = "Keyflex Dev",
  # Reuse the existing release build instead of building again.
  [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$here = $PSScriptRoot

# The Store requires a four-part version whose last part is 0.
$appVersion = (Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json).version
$version = "$appVersion.0"

$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
  Sort-Object { [version]$_.Directory.Parent.Name } | Select-Object -Last 1
if (-not $makeappx) { throw "makeappx.exe not found. Install the Windows SDK (it comes with Visual Studio Build Tools)." }

if (-not $SkipBuild) {
  Push-Location $root
  try {
    npx tauri build --no-bundle
    if ($LASTEXITCODE -ne 0) { throw "The app build failed." }
  } finally { Pop-Location }
}

$exe = Join-Path $root "src-tauri\target\release\keyflex.exe"
if (-not (Test-Path $exe)) { throw "No release build at $exe. Run without -SkipBuild." }

# Lay the package out in a clean folder.
$layout = Join-Path $here "out\layout"
if (Test-Path $layout) { Remove-Item $layout -Recurse -Force }
New-Item -ItemType Directory -Force (Join-Path $layout "Assets") | Out-Null
Copy-Item $exe $layout
foreach ($logo in "StoreLogo.png", "Square44x44Logo.png", "Square150x150Logo.png") {
  Copy-Item (Join-Path $root "src-tauri\icons\$logo") (Join-Path $layout "Assets")
}

# XML-escape the values, since publisher names can contain characters like &.
$escape = { param($text) [System.Security.SecurityElement]::Escape($text) }
$manifest = (Get-Content (Join-Path $here "AppxManifest.xml") -Raw).
  Replace("{{IDENTITY_NAME}}", (& $escape $IdentityName)).
  Replace("{{PUBLISHER}}", (& $escape $Publisher)).
  Replace("{{PUBLISHER_DISPLAY_NAME}}", (& $escape $PublisherDisplayName)).
  Replace("{{VERSION}}", $version)
[IO.File]::WriteAllText((Join-Path $layout "AppxManifest.xml"), $manifest, (New-Object Text.UTF8Encoding($false)))

$package = Join-Path $here "out\Keyflex_${version}_x64.msix"
& $makeappx.FullName pack /o /d $layout /p $package | Select-Object -Last 3
if ($LASTEXITCODE -ne 0) { throw "makeappx failed." }

"Built $package ($([math]::Round((Get-Item $package).Length / 1MB, 2)) MB)"
if ($IdentityName -eq "Keyflex.Dev") {
  "NOTE: built with placeholder identity values. The Store will reject this one;"
  "      pass -IdentityName, -Publisher and -PublisherDisplayName from Partner Center."
}

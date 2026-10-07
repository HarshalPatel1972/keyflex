# Turns the mood SVGs into the PNGs the app embeds, and a preview sheet.
#
#   python design/moods.py
#   pwsh design/render-moods.ps1
#
# Writes  core/assets/moods/<mood>.png      256 px, for the tip popup
#         src-tauri/icons/tray/<mood>.png    64 px, for the tray icon
#         design/moods-preview.png           every face on one sheet
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$scratch = Join-Path $env:TEMP "keyflex-moods"
New-Item -ItemType Directory -Force (Join-Path $root "src-tauri\icons\tray") | Out-Null

Push-Location $root
try {
  $moods = Get-ChildItem "core\assets\moods\*.svg" | Sort-Object Name
  foreach ($svg in $moods) {
    $name = $svg.BaseName
    $out = Join-Path $scratch $name
    if (Test-Path $out) { Remove-Item $out -Recurse -Force }
    npx tauri icon $svg.FullName -o $out *> $null
    Copy-Item (Join-Path $out "128x128@2x.png") "core\assets\moods\$name.png" -Force
    Copy-Item (Join-Path $out "64x64.png") "src-tauri\icons\tray\$name.png" -Force
  }
  "rendered $($moods.Count) moods"

  # Preview sheet.
  $cells = ($moods | ForEach-Object {
    "<div><img src='file:///$(($_.FullName -replace '\.svg$', '.png').Replace('\', '/'))' width='132'><br>$($_.BaseName)</div>"
  }) -join ""
  $html = "<body style='margin:0;background:#18211c;display:flex;flex-wrap:wrap;gap:14px;padding:18px;font:700 15px Segoe UI;color:#e4ede6;text-align:center'>$cells</body>"
  $page = Join-Path $scratch "sheet.html"
  [IO.File]::WriteAllText($page, $html)
  $edge = "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe"
  if (Test-Path $edge) {
    $rows = [math]::Ceiling($moods.Count / 7)
    & $edge --headless=new --disable-gpu --hide-scrollbars --allow-file-access-from-files "--window-size=1060,$(36 + 176 * $rows)" "--screenshot=$root\design\moods-preview.png" "file:///$($page.Replace('\', '/'))" *> $null
    Start-Sleep -Seconds 3
    "wrote design\moods-preview.png"
  }
} finally { Pop-Location }

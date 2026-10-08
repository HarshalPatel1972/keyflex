# Research helper: opens an app, records what it calls its buttons and menu
# items, and closes it again. Only instances this script started are closed.
#
#   pwsh design/survey.ps1 -Launch mspaint -ProcessName mspaint
#   pwsh design/survey.ps1 -Launch winword -ProcessName WINWORD -Match 'bold|italic|link'
param(
  [Parameter(Mandatory)] [string]$Launch,
  [Parameter(Mandatory)] [string]$ProcessName,
  [string]$Arguments = "",
  # Only print names matching this pattern (ribbons have hundreds of buttons).
  [string]$Match = ".",
  [int]$WaitSeconds = 6,
  [switch]$OpenMenus
)

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$AE = [System.Windows.Automation.AutomationElement]
$CT = [System.Windows.Automation.ControlType]
$Scope = [System.Windows.Automation.TreeScope]
function Is($type) { New-Object System.Windows.Automation.PropertyCondition($AE::ControlTypeProperty, $type) }

$before = @(Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | ForEach-Object Id)
if ($Arguments) { Start-Process $Launch -ArgumentList $Arguments } else { Start-Process $Launch }
Start-Sleep -Seconds $WaitSeconds

try {
  $ids = @(Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | ForEach-Object Id)
  $root = $AE::RootElement.FindAll($Scope::Children, [System.Windows.Automation.Condition]::TrueCondition) |
    Where-Object { $ids -contains $_.Current.ProcessId -and -not $_.Current.BoundingRectangle.IsEmpty -and $_.Current.ClassName -notmatch 'Shell_TrayWnd|Progman|WorkerW' } |
    Sort-Object { $_.Current.BoundingRectangle.Width * $_.Current.BoundingRectangle.Height } -Descending | Select-Object -First 1
  if (-not $root) { "no window found for $ProcessName"; return }
  "== $ProcessName  (window class: $($root.Current.ClassName))"

  $wanted = New-Object System.Windows.Automation.OrCondition((Is $CT::Button), (Is $CT::MenuItem), (Is $CT::SplitButton))
  $found = $root.FindAll($Scope::Descendants, $wanted)
  "   $($found.Count) buttons and menu items; showing those matching '$Match'"
  $found | ForEach-Object {
    $c = $_.Current
    if ($c.Name -match $Match) {
      "   {0,-11} '{1}'  shortcut='{2}'" -f ($c.ControlType.ProgrammaticName -replace 'ControlType\.', ''), $c.Name, $c.AcceleratorKey
    }
  } | Sort-Object -Unique

  if ($OpenMenus) {
    foreach ($menu in ($found | Where-Object { $_.Current.ControlType -eq $CT::MenuItem })) {
      $pattern = $null
      if (-not $menu.TryGetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern, [ref]$pattern)) { continue }
      try {
        $name = $menu.Current.Name
        $pattern.Expand(); Start-Sleep -Milliseconds 1100
        $items = $AE::RootElement.FindAll($Scope::Descendants, (New-Object System.Windows.Automation.AndCondition(
          (New-Object System.Windows.Automation.PropertyCondition($AE::ProcessIdProperty, $root.Current.ProcessId)), (Is $CT::MenuItem))))
        "-- menu '$name': " + (($items | ForEach-Object { $c = $_.Current; if ($c.AcceleratorKey) { "$($c.Name) [$($c.AcceleratorKey)]" } else { $c.Name } } |
          Where-Object { $_ -ne $name } | Select-Object -Unique) -join ' | ')
        $pattern.Collapse(); Start-Sleep -Milliseconds 250
      } catch { "   (could not open '$name')" }
    }
  }
} finally {
  Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | Where-Object { $before -notcontains $_.Id } | Stop-Process -Force -ErrorAction SilentlyContinue
}

# Research helper: what does an app expose to UI Automation?
# Lists the buttons and menu items of a window, and whether each sits inside a
# "document" (web-rendered UI). Content controls (text, tabs, list items) are
# never printed.
#
#   pwsh design/probe.ps1 -ProcessName notepad
#   pwsh design/probe.ps1 -ProcessName Code -Limit 60
param(
  [Parameter(Mandatory)] [string]$ProcessName,
  [int]$Limit = 45,
  [switch]$OpenMenus
)

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$AE = [System.Windows.Automation.AutomationElement]
$CT = [System.Windows.Automation.ControlType]
$Scope = [System.Windows.Automation.TreeScope]
$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker

function In-Document($element) {
  $node = $walker.GetParent($element)
  for ($depth = 0; $node -and $depth -lt 60; $depth++) {
    if ($node.Current.ControlType -eq $CT::Document) { return $true }
    $node = $walker.GetParent($node)
  }
  return $false
}

function Describe($element) {
  $c = $element.Current
  $type = $c.ControlType.ProgrammaticName -replace 'ControlType\.', ''
  $doc = if (In-Document $element) { 'DOC ' } else { '    ' }
  "{0}{1,-11} name='{2}' accel='{3}' id='{4}'" -f $doc, $type, $c.Name, $c.AcceleratorKey, $c.AutomationId
}

# Pick the largest top-level window owned by the process (apps often own helper windows too).
$ids = @(Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | ForEach-Object Id)
$root = $AE::RootElement.FindAll($Scope::Children, [System.Windows.Automation.Condition]::TrueCondition) |
  Where-Object { $ids -contains $_.Current.ProcessId -and -not $_.Current.BoundingRectangle.IsEmpty } |
  Sort-Object { $_.Current.BoundingRectangle.Width * $_.Current.BoundingRectangle.Height } -Descending |
  Where-Object { $_.Current.ClassName -notmatch 'Shell_TrayWnd|Progman|WorkerW' } |
  Select-Object -First 1
if (-not $root) { "no window found for $ProcessName"; return }
$process = Get-Process -Id $root.Current.ProcessId
"== $ProcessName  (window class: $($root.Current.ClassName), framework: $($root.Current.FrameworkId))"

$wanted = New-Object System.Windows.Automation.OrCondition(
  (New-Object System.Windows.Automation.PropertyCondition($AE::ControlTypeProperty, $CT::Button)),
  (New-Object System.Windows.Automation.PropertyCondition($AE::ControlTypeProperty, $CT::MenuItem)),
  (New-Object System.Windows.Automation.PropertyCondition($AE::ControlTypeProperty, $CT::SplitButton)))
$found = $root.FindAll($Scope::Descendants, $wanted)
$documents = $root.FindAll($Scope::Descendants, (New-Object System.Windows.Automation.PropertyCondition($AE::ControlTypeProperty, $CT::Document))).Count
"   buttons/menu items: $($found.Count), documents: $documents"
$found | Select-Object -First $Limit | ForEach-Object { Describe $_ }

if ($OpenMenus) {
  # Expand each top-level menu to read its items, then close it again.
  $menus = $found | Where-Object { $_.Current.ControlType -eq $CT::MenuItem }
  foreach ($menu in $menus) {
    $pattern = $null
    if (-not $menu.TryGetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern, [ref]$pattern)) { continue }
    try {
      $pattern.Expand(); Start-Sleep -Milliseconds 450
      "-- menu '$($menu.Current.Name)'"
      $items = $AE::RootElement.FindAll($Scope::Descendants, (New-Object System.Windows.Automation.AndCondition(
        (New-Object System.Windows.Automation.PropertyCondition($AE::ProcessIdProperty, $process.Id)),
        (New-Object System.Windows.Automation.PropertyCondition($AE::ControlTypeProperty, $CT::MenuItem)))))
      $items | Where-Object { $_.Current.Name -ne $menu.Current.Name } | Select-Object -First 30 | ForEach-Object { Describe $_ }
      $pattern.Collapse(); Start-Sleep -Milliseconds 200
    } catch { "   (could not open: $($_.Exception.Message))" }
  }
}

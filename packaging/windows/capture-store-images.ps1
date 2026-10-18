# Microsoft Store のスクリーンショットを撮る（Issue #88・ADR 0026）。施主の Windows で動かす。
#   pwsh packaging/windows/capture-store-images.ps1 -Exe C:\path\nene-ruler.exe -OutDir C:\path\out
#
# 無地の背景の窓（1920×1080・物理画素）に、架空の画面の絵を描いて出し、その上にルーラーを置いて、背景の窓の範囲だけを撮る。
# 施主のデスクトップは写さない。ルーラーはこのスクリプトが起動した物を PID で扱い、施主が起動した物には触らない。
# 入力は窓へ直に送る（PostMessage）だけで、マウスとキーボードは動かさない。
# ルーラーには空の一時フォルダを APPDATA として渡し、既定の設定（赤のインク）で描かせる。施主の設定ファイルは読まず、書かない。
# 位置の計算は 1x（拡大率 100%〜149%）の窓の形による: 枠の左上の外に 4 px、右下の外にギアの分の 20 px（core の FrameLayout）。
param(
    [Parameter(Mandatory = $true)] [string] $Exe,
    [Parameter(Mandatory = $true)] [string] $OutDir
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class RulerWin32 {
    public delegate bool EnumProc(IntPtr window, IntPtr parameter);
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr value);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr parameter);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassNameW(IntPtr window, System.Text.StringBuilder name, int length);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr window, uint message, IntPtr wparam, IntPtr lparam);
    public static IntPtr Find(uint process, string className) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((window, parameter) => {
            uint owner;
            GetWindowThreadProcessId(window, out owner);
            var name = new System.Text.StringBuilder(64);
            GetClassNameW(window, name, 64);
            if (owner == process && name.ToString() == className && IsWindowVisible(window)) { found = window; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
'@
# 物理画素で座標を扱う（PowerShell は既定で dpi 非対応で、座標が拡大率で直される）。
[void][RulerWin32]::SetProcessDpiAwarenessContext([IntPtr](-4))
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$width = 1920
$height = 1080
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
if ($screen.Width -lt $width -or $screen.Height -lt $height) { throw "主画面が $($screen.Width)×$($screen.Height) で、1920×1080 に足りない" }
$origin = New-Object System.Drawing.Point(($screen.X + [int](($screen.Width - $width) / 2)), ($screen.Y + [int](($screen.Height - $height) / 2)))
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# 架空の画面: 明るい地に、登録の画面のカード（1120×640）と、その中の入力欄（992×56）とボタン（280×64）。
$card = New-Object System.Drawing.Rectangle(160, 200, 1120, 640)
$field = New-Object System.Drawing.Rectangle(($card.X + 64), ($card.Y + 214), 992, 56)
$button = New-Object System.Drawing.Rectangle(($card.X + 64), ($card.Bottom - 136), 280, 64)
$form = New-Object System.Windows.Forms.Form
$form.FormBorderStyle = 'None'
$form.StartPosition = 'Manual'
$form.ShowInTaskbar = $false
$form.Location = $origin
$form.ClientSize = New-Object System.Drawing.Size($width, $height)
$form.BackColor = [System.Drawing.Color]::FromArgb(0xEE, 0xF0, 0xF3)
$form.Add_Paint({
    param($sender, $event)
    $g = $event.Graphics
    $g.SmoothingMode = 'AntiAlias'
    $g.TextRenderingHint = 'ClearTypeGridFit'
    $g.FillRectangle([System.Drawing.Brushes]::White, $card)
    $g.DrawRectangle((New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(0xD6, 0xDA, 0xE0))), $card)
    $title = New-Object System.Drawing.Font('Segoe UI Semibold', 40, [System.Drawing.GraphicsUnit]::Pixel)
    $label = New-Object System.Drawing.Font('Segoe UI', 22, [System.Drawing.GraphicsUnit]::Pixel)
    $ink = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(0x1F, 0x23, 0x28))
    $muted = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(0x6A, 0x73, 0x7D))
    $g.DrawString('Create your account', $title, $ink, ($card.X + 64), ($card.Y + 56))
    $fieldPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(0xC4, 0xC9, 0xD0))
    foreach ($row in 0, 1) {
        $top = $field.Y + $row * 130
        $g.DrawString(@('Email', 'Password')[$row], $label, $muted, $field.X, ($top - 38))
        $g.DrawRectangle($fieldPen, $field.X, $top, $field.Width, $field.Height)
    }
    $g.FillRectangle((New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(0x25, 0x63, 0xEB))), $button)
    $format = New-Object System.Drawing.StringFormat
    $format.Alignment = 'Center'
    $format.LineAlignment = 'Center'
    $g.DrawString('Sign up', $label, [System.Drawing.Brushes]::White, [System.Drawing.RectangleF]$button, $format)
})
$form.Show()
[System.Windows.Forms.Application]::DoEvents()

$settings = Join-Path $OutDir 'settings-tmp'
if (Test-Path -LiteralPath $settings) { Remove-Item -LiteralPath $settings -Recurse -Force }
New-Item -ItemType Directory -Force -Path $settings | Out-Null
$start = New-Object System.Diagnostics.ProcessStartInfo((Resolve-Path $Exe))
$start.UseShellExecute = $false
$start.EnvironmentVariables['APPDATA'] = (Resolve-Path $settings).Path
$ruler = [System.Diagnostics.Process]::Start($start)
try {
    $frame = [IntPtr]::Zero
    for ($i = 0; $i -lt 50 -and $frame -eq [IntPtr]::Zero; $i++) {
        Start-Sleep -Milliseconds 100
        $frame = [RulerWin32]::Find([uint32]$ruler.Id, 'NeNeRulerFrame')
    }
    if ($frame -eq [IntPtr]::Zero) { throw 'ルーラーの枠の窓が見つからない' }
    $topmost = [IntPtr](-1)

    # 枠の内側を矩形 $target にぴったり重ねる（窓は枠の左上の外に 4 px、右下の外に 20 px）。
    function Set-Frame([System.Drawing.Rectangle] $target) {
        [void][RulerWin32]::SetWindowPos($frame, $topmost, ($origin.X + $target.X - 4), ($origin.Y + $target.Y - 4), ($target.Width + 24), ($target.Height + 24), 0x0040)
        Start-Sleep -Milliseconds 400
        [System.Windows.Forms.Application]::DoEvents()
    }
    function Save-Shot([string] $name) {
        Start-Sleep -Milliseconds 400
        $bitmap = New-Object System.Drawing.Bitmap($width, $height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($origin, [System.Drawing.Point]::Empty, $bitmap.Size)
        $path = Join-Path $OutDir $name
        $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
        $graphics.Dispose()
        $bitmap.Dispose()
        Write-Output "shot: $path"
    }

    # 1. カードを測る（全部描いた表示: 目盛と幅・高さ）。
    Set-Frame $card
    Save-Shot 'store-1-measure-card.png'
    # 2. 入力欄を測る（小さな枠: 中央に W × H px）。
    Set-Frame $field
    Save-Shot 'store-2-measure-field.png'
    # 3. カードの枠で、ギアから設定モーダルを開く。ギアは枠の右下の外（窓の座標で (W + 16, H + 16) が中心）。
    Set-Frame $card
    $gear = (($card.Height + 16) -shl 16) -bor ($card.Width + 16)
    [void][RulerWin32]::PostMessageW($frame, 0x0201, [IntPtr]1, [IntPtr]$gear)
    [void][RulerWin32]::PostMessageW($frame, 0x0202, [IntPtr]0, [IntPtr]$gear)
    Start-Sleep -Milliseconds 800
    Save-Shot 'store-3-settings.png'
}
finally {
    if (-not $ruler.HasExited) { Stop-Process -Id $ruler.Id; $ruler.WaitForExit() }
    $form.Close()
    Remove-Item -LiteralPath $settings -Recurse -Force
}

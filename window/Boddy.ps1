param([switch]$SmokeTest, [string]$PreviewPath)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[Windows.Forms.Application]::EnableVisualStyles()
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$binary = Join-Path $projectRoot 'target\debug\boddy-cli.exe'
$script:history = New-Object 'System.Collections.Generic.List[object]'
$script:active = $null
$script:lastReply = ''
$form = New-Object Windows.Forms.Form
$form.Text = 'Boddy 1.0'
$form.ClientSize = New-Object Drawing.Size(900, 680)
$form.MinimumSize = New-Object Drawing.Size(650, 480)
$form.StartPosition = 'CenterScreen'
$form.BackColor = [Drawing.ColorTranslator]::FromHtml('#17191D')
$form.ForeColor = [Drawing.Color]::WhiteSmoke
$form.Font = New-Object Drawing.Font('Segoe UI', 11)
$title = New-Object Windows.Forms.Label
$title.Text = 'Boddy 1.0'
$title.Font = New-Object Drawing.Font('Segoe UI Semibold', 24)
$title.SetBounds(30, 20, 350, 50)
$subtitle = New-Object Windows.Forms.Label
$subtitle.Text = 'Tu espacio para pensar, aprender y crear.'
$subtitle.ForeColor = [Drawing.Color]::Silver
$subtitle.SetBounds(32, 76, 650, 28)
$newChat = New-Object Windows.Forms.Button
$newChat.Text = 'Nuevo chat'
$newChat.SetBounds(728, 32, 140, 36)
$newChat.Anchor = 'Top,Right'
$newChat.FlatStyle = 'Flat'
$newChat.FlatAppearance.BorderColor = [Drawing.Color]::DimGray
$transcript = New-Object Windows.Forms.RichTextBox
$transcript.SetBounds(32, 125, 835, 365)
$transcript.Anchor = 'Top,Bottom,Left,Right'
$transcript.ReadOnly = $true
$transcript.BorderStyle = 'None'
$transcript.BackColor = $form.BackColor
$transcript.ForeColor = $form.ForeColor
$transcript.Font = New-Object Drawing.Font('Segoe UI', 12)
$transcript.DetectUrls = $true
$transcript.Text = "Boddy`nHola, soy Boddy. Escribe lo que quieres explorar o construir.`n`n"
$inputPanel = New-Object Windows.Forms.Panel
$inputPanel.SetBounds(30, 510, 838, 105)
$inputPanel.Anchor = 'Bottom,Left,Right'
$inputPanel.BackColor = [Drawing.ColorTranslator]::FromHtml('#282B31')
$prompt = New-Object Windows.Forms.TextBox
$prompt.SetBounds(16, 15, 655, 75)
$prompt.Anchor = 'Top,Bottom,Left,Right'
$prompt.Multiline = $true
$prompt.AcceptsReturn = $true
$prompt.MaxLength = 8000
$prompt.BorderStyle = 'None'
$prompt.BackColor = $inputPanel.BackColor
$prompt.ForeColor = [Drawing.Color]::WhiteSmoke
$prompt.Font = New-Object Drawing.Font('Segoe UI', 12)
$send = New-Object Windows.Forms.Button
$send.Text = 'Enviar'
$send.SetBounds(694, 33, 125, 40)
$send.Anchor = 'Bottom,Right'
$send.FlatStyle = 'Flat'
$send.FlatAppearance.BorderSize = 0
$send.BackColor = [Drawing.ColorTranslator]::FromHtml('#80DDC7')
$send.ForeColor = [Drawing.ColorTranslator]::FromHtml('#14251F')
$inputPanel.Controls.AddRange(@($prompt, $send))
$status = New-Object Windows.Forms.Label
$status.Text = 'Modelo local: Qwen2.5-Coder 7B | Enter para enviar; Shift+Enter para nueva linea'
$status.Font = New-Object Drawing.Font('Segoe UI', 9)
$status.ForeColor = [Drawing.Color]::Silver
$status.SetBounds(32, 633, 835, 25)
$status.Anchor = 'Bottom,Left,Right'
$form.Controls.AddRange(@($title, $subtitle, $newChat, $transcript, $inputPanel, $status))
$timer = New-Object Windows.Forms.Timer
$timer.Interval = 80
function Add-ChatText([string]$role, [string]$text) {
    $transcript.SelectionStart = $transcript.TextLength
    $transcript.SelectionColor = if ($role -eq 'Tu') { [Drawing.ColorTranslator]::FromHtml('#80DDC7') } else { [Drawing.Color]::WhiteSmoke }
    $transcript.AppendText($role + "`n")
    $transcript.SelectionColor = [Drawing.Color]::Gainsboro
    $transcript.AppendText($text + "`n`n")
    $transcript.ScrollToCaret()
}
function Reset-Request {
    $timer.Stop()
    if ($script:active) { $script:active.Dispose(); $script:active = $null }
    $prompt.Enabled = $true
    $newChat.Enabled = $true
    $send.Text = 'Enviar'
    $prompt.Focus() | Out-Null
}
$send.Add_Click({
    if ($script:active) {
        if (-not $script:active.HasExited) { $script:active.Kill() }
        if ($script:history.Count) { $script:history.RemoveAt($script:history.Count - 1) }
        Add-ChatText 'Boddy' 'Respuesta cancelada.'
        Reset-Request
        $status.Text = 'Cancelado. Puedes continuar.'
        return
    }
    $text = $prompt.Text.Trim()
    if (-not $text) { return }
    try {
        if (-not (Test-Path -LiteralPath $binary)) { throw 'Ejecuta primero window/Start-Boddy.ps1.' }
        # Conservar pares de conversacion dentro del presupuesto del host.
        while ($script:history.Count -gt 12 -or (($script:history | ForEach-Object { $_.content.Length } | Measure-Object -Sum).Sum + $text.Length) -gt 15000) {
            if ($script:history.Count -lt 2) { break }
            $script:history.RemoveRange(0,2)
        }
        $script:history.Add(@{role='user';content=$text})
        $payload = @{messages=@($script:history.ToArray())} | ConvertTo-Json -Depth 8 -Compress
        $info = New-Object Diagnostics.ProcessStartInfo
        $info.FileName = $binary
        $info.Arguments = 'chat'
        $info.WorkingDirectory = $projectRoot
        $info.UseShellExecute = $false
        $info.CreateNoWindow = $true
        $info.RedirectStandardInput = $true
        $info.RedirectStandardOutput = $true
        $info.RedirectStandardError = $true
        $info.StandardOutputEncoding = [Text.Encoding]::UTF8
        $info.StandardErrorEncoding = [Text.Encoding]::UTF8
        $script:active = New-Object Diagnostics.Process
        $script:active.StartInfo = $info
        [void]$script:active.Start()
        $script:stdout = $script:active.StandardOutput.ReadToEndAsync()
        $script:stderr = $script:active.StandardError.ReadToEndAsync()
        $bytes = [Text.Encoding]::UTF8.GetBytes($payload)
        $script:active.StandardInput.BaseStream.Write($bytes,0,$bytes.Length)
        $script:active.StandardInput.Close()
        $script:started = [DateTime]::UtcNow
        Add-ChatText 'Tu' $text
        $prompt.Clear()
        $prompt.Enabled = $false
        $newChat.Enabled = $false
        $send.Text = 'Cancelar'
        $status.Text = 'Boddy esta pensando...'
        $timer.Start()
    } catch {
        if ($script:history.Count -and $script:history[$script:history.Count-1].role -eq 'user') { $script:history.RemoveAt($script:history.Count-1) }
        Add-ChatText 'Sistema' $_.Exception.Message
        Reset-Request
    }
})
$timer.Add_Tick({
    if (-not $script:active) { return }
    if (([DateTime]::UtcNow - $script:started).TotalSeconds -gt 130) { $send.PerformClick(); return }
    if ($script:active.HasExited -and $script:stdout.IsCompleted -and $script:stderr.IsCompleted) {
        try {
            if ($script:active.ExitCode -ne 0) { throw $script:stderr.Result }
            $reply = $script:stdout.Result | ConvertFrom-Json
            if (-not $reply.content) { throw 'El motor no devolvio texto.' }
            $script:lastReply = [string]$reply.content
            $script:history.Add(@{role='assistant';content=$script:lastReply})
            Add-ChatText 'Boddy' $script:lastReply
            $status.Text = 'Local | Qwen2.5-Coder 7B Q4_K_M | Respuesta lista'
        } catch {
            if ($script:history.Count -and $script:history[$script:history.Count-1].role -eq 'user') { $script:history.RemoveAt($script:history.Count-1) }
            Add-ChatText 'Sistema' $_.Exception.Message
            $status.Text = 'No se pudo obtener una respuesta. Comprueba el motor local.'
        }
        Reset-Request
    }
})
$prompt.Add_KeyDown({
    if ($_.KeyCode -eq 'Enter' -and -not $_.Shift) { $_.SuppressKeyPress = $true; $send.PerformClick() }
})
$newChat.Add_Click({ $script:history.Clear(); $transcript.Clear(); Add-ChatText 'Boddy' 'Nuevo chat. Que quieres construir?'; $prompt.Focus() })
$form.Add_FormClosed({ $timer.Stop(); $timer.Dispose(); if ($script:active) { if (-not $script:active.HasExited) { $script:active.Kill() }; $script:active.Dispose() } })
if ($SmokeTest) {
    $form.Opacity = 0
    $form.Show()
    $prompt.Text = 'Saluda en espanol en una sola frase.'
    $send.PerformClick()
    $deadline = [DateTime]::UtcNow.AddSeconds(140)
    while ($script:active -and [DateTime]::UtcNow -lt $deadline) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25 }
    if (-not $script:lastReply) { throw ('Fallo chat real: ' + $transcript.Text) }
    if ($PreviewPath) {
        $bitmap = New-Object Drawing.Bitmap($form.Width, $form.Height)
        $form.DrawToBitmap($bitmap, (New-Object Drawing.Rectangle(0,0,$form.Width,$form.Height)))
        $bitmap.Save($PreviewPath, [Drawing.Imaging.ImageFormat]::Png)
        $bitmap.Dispose()
    }
    'Boddy 1.0 chat real: ' + $script:lastReply
    $form.Close()
    $form.Dispose()
} else { [void]$form.ShowDialog(); $form.Dispose() }

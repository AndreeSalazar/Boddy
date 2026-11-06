param([switch]$SmokeTest)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$binary = Join-Path $projectRoot 'target\debug\boddy-cli.exe'
$form = New-Object Windows.Forms.Form
$form.Text = 'Boddy | Comandos'
$form.Size = New-Object Drawing.Size(880, 620)
$form.MinimumSize = New-Object Drawing.Size(650, 420)
$form.StartPosition = 'CenterScreen'
$commands = New-Object Windows.Forms.ComboBox
$commands.Location = New-Object Drawing.Point(16, 16)
$commands.Size = New-Object Drawing.Size(160, 28)
$commands.DropDownStyle = 'DropDownList'
$commands.Items.AddRange(@('proposito', 'fuentes', 'demo', 'grafo'))
$commands.SelectedIndex = 0
$parameter = New-Object Windows.Forms.TextBox
$parameter.Location = New-Object Drawing.Point(190, 16)
$parameter.Size = New-Object Drawing.Size(420, 28)
$parameter.Anchor = 'Top,Left,Right'
$run = New-Object Windows.Forms.Button
$run.Text = 'Ejecutar'
$run.Location = New-Object Drawing.Point(630, 14)
$run.Size = New-Object Drawing.Size(110, 30)
$run.Anchor = 'Top,Right'
$output = New-Object Windows.Forms.TextBox
$output.Multiline = $true
$output.ReadOnly = $true
$output.ScrollBars = 'Both'
$output.WordWrap = $false
$output.Location = New-Object Drawing.Point(16, 60)
$output.Size = New-Object Drawing.Size(830, 450)
$output.Anchor = 'Top,Bottom,Left,Right'
$output.Font = New-Object Drawing.Font('Consolas', 10)
$output.Text = 'Control de comandos local. Modelo y sandbox pendientes. Fuentes: parametro de busqueda. Grafo: resumen, fallos o ciclos.'
$status = New-Object Windows.Forms.Label
$status.Text = 'Listo | Sin shell de comandos arbitrarios'
$status.Location = New-Object Drawing.Point(16, 530)
$status.Size = New-Object Drawing.Size(830, 24)
$status.Anchor = 'Bottom,Left,Right'
$form.Controls.AddRange(@($commands, $parameter, $run, $output, $status))
$script:active = $null
$timer = New-Object Windows.Forms.Timer
$timer.Interval = 100
$run.Add_Click({
    try {
        if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw 'Compila antes: cargo build -p boddy-cli --offline' }
        $command = [string]$commands.SelectedItem
        $argument = $parameter.Text.Trim()
        if ($argument.Length -gt 256 -or $argument.Contains('"') -or $argument.Contains("`n") -or $argument.Contains("`r") -or $argument.EndsWith('\')) { throw 'Parametro invalido: maximo 256 caracteres, sin comillas, saltos o barra final.' }
        if ($command -eq 'grafo' -and $argument -notin @('', 'resumen', 'fallos', 'ciclos')) { throw 'Grafo permite resumen, fallos o ciclos.' }
        if ($command -in @('proposito', 'demo') -and $argument) { throw 'Este comando no recibe parametros.' }
        $info = New-Object Diagnostics.ProcessStartInfo
        $info.FileName = $binary
        $info.WorkingDirectory = $projectRoot
        $info.UseShellExecute = $false
        $info.CreateNoWindow = $true
        $info.RedirectStandardOutput = $true
        $info.RedirectStandardError = $true
        $info.StandardOutputEncoding = [Text.Encoding]::UTF8
        $info.StandardErrorEncoding = [Text.Encoding]::UTF8
        $info.Arguments = $command
        if ($argument) { $info.Arguments += ' "' + $argument + '"' }
        $script:active = New-Object Diagnostics.Process
        $script:active.StartInfo = $info
        [void]$script:active.Start()
        $script:stdout = $script:active.StandardOutput.ReadToEndAsync()
        $script:stderr = $script:active.StandardError.ReadToEndAsync()
        $run.Enabled = $false
        $status.Text = 'Ejecutando ' + $command
        $timer.Start()
    } catch { $output.Text = $_.Exception.Message }
})
$timer.Add_Tick({
    if ($script:active -and $script:active.HasExited -and $script:stdout.IsCompleted -and $script:stderr.IsCompleted) {
        $timer.Stop()
        $output.Text = $script:stdout.Result + $script:stderr.Result
        $status.Text = 'Codigo de salida: ' + $script:active.ExitCode
        $script:active.Dispose()
        $script:active = $null
        $run.Enabled = $true
    }
})
$form.Add_FormClosed({
    $timer.Stop()
    $timer.Dispose()
    if ($script:active) { if (-not $script:active.HasExited) { $script:active.Kill() }; $script:active.Dispose() }
})
if ($SmokeTest) {
    if ($form.Controls.Count -ne 5) { throw 'Controles incompletos' }
    # Ejercita el mismo evento que utiliza el boton, sin abrir una ventana.
    $commands.SelectedItem = 'fuentes'
    $parameter.Text = 'python'
    $form.Show()
    $run.PerformClick()
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    while (-not $run.Enabled -and [DateTime]::UtcNow -lt $deadline) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25 }
    if (-not $run.Enabled -or $output.Text -notmatch 'P01') { throw ('Fallo en comando de ventana: ' + $output.Text) }
    $form.Close()
    $form.Dispose()
    'Window smoke test: controles y consulta Python correctos.'
} else { [void]$form.ShowDialog(); $form.Dispose() }

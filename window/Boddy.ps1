param([switch]$SmokeTest, [switch]$ResearchSmokeTest, [switch]$WorkbenchSmokeTest, [switch]$LabSmokeTest, [switch]$AttentionSmokeTest, [switch]$RustSchoolSmokeTest, [switch]$LibrarySmokeTest, [string]$PreviewPath, [string]$EngineStatus)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[Windows.Forms.Application]::EnableVisualStyles()
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$binary = Join-Path $projectRoot 'target\debug\boddy-cli.exe'
$script:history = New-Object 'System.Collections.Generic.List[object]'
$script:active = $null
$script:lastReply = ''
$script:totalInput = 0L
$script:totalOutput = 0L
$script:totalCached = 0L
$script:requestMode = 'chat'
$script:lastTaskState = ''
$form = New-Object Windows.Forms.Form
$form.Text = 'Boddy 1.0'
$form.ClientSize = New-Object Drawing.Size(900, 850)
$form.MinimumSize = New-Object Drawing.Size(760, 650)
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
$transcript.SetBounds(32, 190, 835, 345)
$transcript.Anchor = 'Top,Bottom,Left,Right'
$transcript.ReadOnly = $true
$transcript.BorderStyle = 'None'
$transcript.BackColor = $form.BackColor
$transcript.ForeColor = $form.ForeColor
$transcript.Font = New-Object Drawing.Font('Segoe UI', 12)
$transcript.DetectUrls = $true
$transcript.Text = "Boddy`nHola, soy Boddy. Escribe lo que quieres explorar o construir.`n`n"
$inputPanel = New-Object Windows.Forms.Panel
$inputPanel.SetBounds(30, 550, 838, 105)
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
if($EngineStatus){$status.Text=$EngineStatus}
$status.Font = New-Object Drawing.Font('Segoe UI', 9)
$status.ForeColor = [Drawing.Color]::Silver
$status.SetBounds(32, 668, 835, 25)
$status.Anchor = 'Bottom,Left,Right'
$form.Controls.AddRange(@($title, $subtitle, $newChat, $transcript, $inputPanel, $status))
$contextLabel = New-Object Windows.Forms.Label
$contextLabel.Text = 'Presupuesto de contexto'
$contextLabel.SetBounds(32, 117, 205, 25)
$contextBudget = New-Object Windows.Forms.ComboBox
$contextBudget.DropDownStyle = 'DropDownList'
$contextBudget.Items.AddRange(@('512','1024','2048','4096','8192'))
$contextBudget.SelectedItem = '2048'
$contextBudget.SetBounds(237, 112, 100, 30)
$responseLabel = New-Object Windows.Forms.Label
$responseLabel.Text = 'Respuesta maxima'
$responseLabel.SetBounds(360, 117, 165, 25)
$responseBudget = New-Object Windows.Forms.ComboBox
$responseBudget.DropDownStyle = 'DropDownList'
$responseBudget.Items.AddRange(@('64','128','256','512','1024'))
$responseBudget.SelectedItem = '256'
$responseBudget.SetBounds(525, 112, 90, 30)
$useGraph = New-Object Windows.Forms.CheckBox
$useGraph.Text = 'Mapa de Boddy'
$useGraph.SetBounds(650, 112, 210, 30)
$hint = New-Object Windows.Forms.Label
$hint.Text = 'Tokens: el host recorta turnos antiguos para respetar el presupuesto. El mapa aporta solo cifras.'
$hint.ForeColor = [Drawing.Color]::Silver
$hint.Font = New-Object Drawing.Font('Segoe UI', 9)
$hint.SetBounds(32, 152, 565, 30)
$research = New-Object Windows.Forms.CheckBox
$research.Text = 'Investigar proyecto (lectura)'
$research.SetBounds(600, 148, 270, 30)
$tokenStats = New-Object Windows.Forms.Label
$tokenStats.Text = 'Tokens: envia un mensaje para medir entrada, salida y cache.'
$tokenStats.ForeColor = [Drawing.ColorTranslator]::FromHtml('#80DDC7')
$tokenStats.Font = New-Object Drawing.Font('Segoe UI', 9)
$tokenStats.SetBounds(32, 701, 835, 52)
$tokenStats.Anchor = 'Bottom,Left,Right'
$gpuStats = New-Object Windows.Forms.Label
$gpuStats.Text = 'GPU: se muestra una muestra de VRAM del dispositivo al terminar cada respuesta.'
$gpuStats.ForeColor = [Drawing.Color]::Silver
$gpuStats.Font = New-Object Drawing.Font('Segoe UI', 9)
$gpuStats.SetBounds(32, 760, 835, 24)
$gpuStats.Anchor = 'Bottom,Left,Right'
$form.Controls.AddRange(@($contextLabel,$contextBudget,$responseLabel,$responseBudget,$useGraph,$hint,$tokenStats,$gpuStats,$research))
$research.Add_CheckedChanged({
    $contextBudget.Enabled = -not $research.Checked
    $responseBudget.Enabled = -not $research.Checked
    $useGraph.Enabled = -not $research.Checked
    $hint.Text = if ($research.Checked) { 'Lectura: hasta 3 archivos, 4 llamadas al modelo y 1 correccion; objetivo de hasta 800 caracteres.' } else { 'Tokens: recorte por presupuesto. El mapa aporta solo cifras.' }
})
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
    $research.Enabled = $true
    $contextBudget.Enabled = -not $research.Checked
    $responseBudget.Enabled = -not $research.Checked
    $useGraph.Enabled = -not $research.Checked
    $send.Text = 'Enviar'
    if(Get-Command Update-ModelControls -ErrorAction SilentlyContinue){Update-ModelControls}
    $prompt.Focus() | Out-Null
}
$send.Add_Click({
    if ($script:active) {
        if (-not $script:active.HasExited) { $script:active.Kill() }
        if ($script:requestMode -eq 'chat' -and $script:history.Count) { $script:history.RemoveAt($script:history.Count - 1) }
        Add-ChatText 'Boddy' 'Respuesta cancelada.'
        Reset-Request
        $status.Text = 'Cancelado. Puedes continuar.'
        return
    }
    $text = $prompt.Text.Trim()
    if (-not $text) { return }
    $tiny=$chatModels.SelectedItem.id -ne 'qwen-local'
    if($script:wbProcess -and $script:wbCommand -eq 'entrenar') { $status.Text='Espera a que termine el entrenamiento GPU.'; return }
    $script:requestMode = if ($tiny) { 'modelo' } elseif ($research.Checked) { 'investigar' } else { 'chat' }
    try {
        if (-not (Test-Path -LiteralPath $binary)) { throw 'Ejecuta primero window/Start-Boddy.ps1.' }
        if ($tiny) {
            if($text.Length -gt 256 -or [int]$responseBudget.SelectedItem -gt 256){throw 'Tiny admite un prefijo ASCII y salida de hasta 256 caracteres.'}
            $payload=@{action='generate';id=$chatModels.SelectedItem.id;prefix=$text;max_tokens=[int]$responseBudget.SelectedItem} | ConvertTo-Json -Compress
        } elseif ($research.Checked) {
            if ($text.Length -gt 800) { throw 'El objetivo de lectura admite hasta 800 caracteres.' }
            $payload = @{objective=$text} | ConvertTo-Json -Compress
        } else {
        # Limites de transporte adicionales; el host calcula el presupuesto real en tokens.
        if ($script:history.Count -ge 24 -or (($script:history | ForEach-Object { $_.content.Length } | Measure-Object -Sum).Sum + $text.Length) -gt 16000) {
            throw 'El historial supera el limite de transporte. Inicia un nuevo chat o acorta el mensaje.'
        }
        $script:history.Add(@{role='user';content=$text})
        $payload = @{messages=@($script:history.ToArray()); context_tokens=[int]$contextBudget.SelectedItem; max_tokens=[int]$responseBudget.SelectedItem; use_graph=$useGraph.Checked} | ConvertTo-Json -Depth 8 -Compress
        }
        $info = New-Object Diagnostics.ProcessStartInfo
        $info.FileName = $binary
        $info.Arguments = $script:requestMode
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
        $contextBudget.Enabled = $false
        $responseBudget.Enabled = $false
        $useGraph.Enabled = $false
        $research.Enabled = $false
        $chatModels.Enabled=$false; $chatReload.Enabled=$false
        $send.Text = 'Cancelar'
        $status.Text = if ($research.Checked) { 'Boddy esta investigando el proyecto (solo lectura)...' } else { 'Boddy esta pensando...' }
        $timer.Start()
    } catch {
        if ($script:requestMode -eq 'chat' -and $script:history.Count -and $script:history[$script:history.Count-1].role -eq 'user') { $script:history.RemoveAt($script:history.Count-1) }
        Add-ChatText 'Sistema' $_.Exception.Message
        Reset-Request
    }
})
$timer.Add_Tick({
    if (-not $script:active) { return }
    $timeout = if ($script:requestMode -eq 'investigar') { 420 } else { 130 }
    if (([DateTime]::UtcNow - $script:started).TotalSeconds -gt $timeout) { $send.PerformClick(); return }
    if ($script:active.HasExited -and $script:stdout.IsCompleted -and $script:stderr.IsCompleted) {
        try {
            if ($script:active.ExitCode -ne 0) { throw $script:stderr.Result }
            $reply = $script:stdout.Result | ConvertFrom-Json
            if (-not $reply.content) { throw 'El motor no devolvio texto.' }
            $script:lastReply = [string]$reply.content
            if ($script:requestMode -eq 'chat') {
            $script:history.Clear()
            foreach ($message in $reply.context_messages) { $script:history.Add(@{role=[string]$message.role; content=[string]$message.content}) }
            $script:history.Add(@{role='assistant';content=$script:lastReply})
            } elseif ($script:requestMode -eq 'investigar') { $script:lastTaskState=[string]$reply.task.state; $script:lastReply += "`nRegistro: " + $reply.journal_path }
            $speaker=if($script:requestMode -eq 'modelo'){'Boddy Tiny | continuacion experimental'}else{'Boddy'}
            Add-ChatText $speaker $script:lastReply
            if ($null -ne $reply.usage.prompt_tokens -and $null -ne $reply.usage.completion_tokens) {
                $inputTokens = [long]$reply.usage.prompt_tokens
                $outputTokens = [long]$reply.usage.completion_tokens
                $cachedTokens = [long]$reply.usage.prompt_tokens_details.cached_tokens
                $script:totalInput += $inputTokens
                $script:totalOutput += $outputTokens
                $script:totalCached += $cachedTokens
                $speed = if ($reply.timings.predicted_per_second) { '{0:N1} tok/s' -f [double]$reply.timings.predicted_per_second } else { 'velocidad de generacion no informada' }
                $tokenStats.Text = "Ultima: entrada $inputTokens (cache $cachedTokens), salida $outputTokens | $speed | {0:N1} s`nAcumulado: entrada {1}, salida {2}, cache {3} | Recorte: {4} mensajes / {5} tokens de entrada" -f ([double]$reply.elapsed_ms/1000),$script:totalInput,$script:totalOutput,$script:totalCached,$reply.budget.dropped_messages,$reply.budget.saved_input_tokens
                $contextDisplay = [string]($inputTokens+$outputTokens)
            } else { $tokenStats.Text = 'El motor no informo uso de tokens; no se agrego al acumulado.'; $contextDisplay='n/d' }
            if ($reply.gpu.available) {
                $gpuStats.Text = "GPU total: {0:N2} / {1:N2} GiB | Uso {2}% | Muestra al responder; incluye escritorio, no es pico" -f ([double]$reply.gpu.used_mib/1024),([double]$reply.gpu.total_mib/1024),$reply.gpu.utilization_percent
            } else { $gpuStats.Text = 'GPU: lectura no disponible. No se supone un consumo de cero.' }
            $status.Text = 'Contexto {0}/{1} tokens | Limite configurado del motor: {2} | Mapa: {3}' -f $contextDisplay,$reply.budget.context_tokens,$reply.budget.server_capacity,$reply.graph_context_used
            if ($reply.finish_reason -eq 'length') { $status.Text += ' | Respuesta cortada por limite' }
            if ($script:requestMode -eq 'investigar') {
                $status.Text = 'Tarea: {0} | Modelo: {1}/4 | Herramientas: {2}/6 | Correcciones: {3}/1' -f $reply.task.state,$reply.task.model_calls,$reply.task.tool_calls,$reply.task.repairs_used
                $tokenStats.Text = "Investigacion: entrada $inputTokens, salida $outputTokens, cache $cachedTokens | {0:N1} s`nAcumulado de esta ventana: entrada {1}, salida {2}, cache {3}" -f ([double]$reply.elapsed_ms/1000),$script:totalInput,$script:totalOutput,$script:totalCached
            }
            if($script:requestMode -eq 'modelo'){
                $status.Text='Tiny: '+$reply.id+' | Continuacion de caracteres; no es respuesta a instrucciones'
                $tokenStats.Text='Contexto utilizado: '+$reply.context_tokens+' caracteres | Generados: '+$reply.completion_tokens+' | Sin cache ni historial'
                $gpuStats.Text='Generacion: CPU de referencia. El entrenamiento Tiny usa WGPU.'
            }
        } catch {
            if ($script:requestMode -eq 'chat' -and $script:history.Count -and $script:history[$script:history.Count-1].role -eq 'user') { $script:history.RemoveAt($script:history.Count-1) }
            Add-ChatText 'Sistema' $_.Exception.Message
            $status.Text = 'No se pudo obtener una respuesta. Comprueba el motor local.'
        }
        Reset-Request
    }
})
$prompt.Add_KeyDown({
    if ($_.KeyCode -eq 'Enter' -and -not $_.Shift) { $_.SuppressKeyPress = $true; $send.PerformClick() }
})
$newChat.Add_Click({ $script:history.Clear(); $script:totalInput=0L; $script:totalOutput=0L; $script:totalCached=0L; $tokenStats.Text='Nuevo chat: acumulados reiniciados.'; $transcript.Clear(); Add-ChatText 'Boddy' 'Nuevo chat. Que quieres construir?'; $prompt.Focus() })
$form.Add_FormClosed({ $timer.Stop(); $timer.Dispose(); if ($script:active) { if (-not $script:active.HasExited) { $script:active.Kill() }; $script:active.Dispose() } })
. (Join-Path $PSScriptRoot 'components\Workbench.ps1')
. (Join-Path $PSScriptRoot 'components\Laboratory.ps1')
. (Join-Path $PSScriptRoot 'components\ModelSelector.ps1')
. (Join-Path $PSScriptRoot 'components\Attention.ps1')
. (Join-Path $PSScriptRoot 'components\RustSchool.ps1')
. (Join-Path $PSScriptRoot 'components\Library.ps1')
if($LibrarySmokeTest){
    function Wait-LibrarySmoke { $deadline=[DateTime]::UtcNow.AddSeconds(20);while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 25};if($script:wbProcess){throw 'Tiempo de espera de biblioteca agotado.'} }
    $form.Opacity=0;$form.Show();[Windows.Forms.Application]::DoEvents();Wait-LibrarySmoke
    $tabs.SelectedTab=$libraryPage;$libraryCategories.PerformClick();Wait-LibrarySmoke
    if($libraryCategory.Items.Count -ne 6){throw 'No se cargaron las categorias desde el host.'}
    $librarySearch.PerformClick();Wait-LibrarySmoke
    if($libraryCards.Items.Count -lt 1 -or $script:wbLastResult.documents_read -ne 0){throw ('Busqueda documental incorrecta: '+$wbMessage.Text)}
    $libraryBudget.SelectedItem='512';$libraryContext.PerformClick();Wait-LibrarySmoke
    if($script:wbLastResult.context_bytes -gt 512 -or -not $script:wbLastResult.references){throw ('Contexto fuera de presupuesto: '+$wbMessage.Text)}
    $libraryRemember.PerformClick();Wait-LibrarySmoke
    if(-not $script:wbLastResult.memory_id){throw 'No se guardo el recuerdo.'}
    $libraryRecall.PerformClick();Wait-LibrarySmoke
    if(-not $script:wbLastResult.reread){throw 'No se releyeron referencias.'}
    $libraryExport.PerformClick();Wait-LibrarySmoke
    if(-not (Test-Path -LiteralPath (Join-Path $projectRoot $script:wbLastResult.markdown_path))){throw 'No se exporto Markdown.'}
    $libraryLoadMemory.PerformClick();Wait-LibrarySmoke
    if($null -eq $script:wbLastResult.memories -or $libraryMemory.Items.Count -lt 1){throw ('No se recupero la lista de recuerdos: '+$wbMessage.Text)}
    $libraryQuery.Text='es';$libraryLanguages.PerformClick();Wait-LibrarySmoke
    if($script:wbLastResult.records -lt 7000 -or $script:history.Count -ne 0){throw 'Fallo catalogo o se altero el chat.'}
    'Biblioteca: registro, contexto de 512 bytes, recuerdo, relectura, exportacion y catalogo IANA desde ventana.'
    $form.Close();$form.Dispose()
} elseif($RustSchoolSmokeTest){
    $form.Opacity=0;$form.Show();[Windows.Forms.Application]::DoEvents()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 25}
    $tabs.SelectedTab=$rustPage;$rustTrain.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(180)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 25}
    if($script:wbLastResult.training_profile -ne 'rust-v1'){throw ('Fallo entrenamiento Rust UI: '+$wbMessage.Text)}
    $rustRun.PerformClick();$deadline=[DateTime]::UtcNow.AddSeconds(180)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 25}
    if($script:wbLastResult.task_count -ne 6 -or $script:wbLastResult.policy.execute_code -or $rustOutput.Text -notmatch 'PRIMERA ELECCION'){throw ('Fallo agente Rust UI: '+$wbMessage.Text)}
    if($script:history.Count -ne 0){throw 'La escuela altero el historial del chat.'}
    'Escuela Rust: entrenamiento GPU y ciclo de verificacion desde ventana; estado '+$script:wbLastResult.state
    $form.Close();$form.Dispose()
} elseif($AttentionSmokeTest){
    $form.Opacity=0;$form.Show();[Windows.Forms.Application]::DoEvents()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 25}
    $tabs.SelectedTab=$attentionPage;$attentionTrain.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(180)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 25}
    if(-not $script:wbLastResult.checkpoint_roundtrip -or $script:wbLastResult.model -ne 'Boddy Attention 0.1'){throw ('Fallo Attention: '+$wbMessage.Text)}
    if($script:wbLastResult.loss.train_after -ge $script:wbLastResult.loss.train_before){throw 'No bajo la perdida Attention.'}
    $parent=$script:wbLastResult.id;$attentionSteps.SelectedItem='100';$attentionTrain.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(180)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 25}
    if($script:wbLastResult.parent_id -ne $parent -or $script:wbLastResult.completed_steps -ne 400){throw 'No continuo Attention.'}
    $unicodePrefix=([char]0x00bf)+'Qu'+([char]0x00e9)+' tal? '+[char]::ConvertFromUtf32(0x1f9e0)
    $attentionPrefix.Text=$unicodePrefix;$attentionGenerate.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(30)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 25}
    if(-not $script:wbLastResult.content.StartsWith($unicodePrefix) -or $script:history.Count -ne 0){throw 'Fallo texto Unicode Attention.'}
    'Attention: entrenamiento 300, continuacion a 400 y generacion Unicode desde ventana.'
    $form.Close();$form.Dispose()
} elseif ($LabSmokeTest) {
    $form.Opacity=0; $form.Show()
    $tabs.SelectedTab=$labPage
    [Windows.Forms.Application]::DoEvents()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    $labSteps.SelectedItem='1000'; $labTrain.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(190)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if(-not $script:wbLastResult.checkpoint_roundtrip -or $labOutput.Text -notmatch 'NVIDIA GeForce RTX 3060') {throw ('Fallo laboratorio: '+$wbMessage.Text+' | '+$labOutput.Text)}
    if($script:wbLastResult.loss.train_after -ge $script:wbLastResult.loss.train_before){throw 'No bajo la perdida de entrenamiento.'}
    if($script:history.Count -ne 0){throw 'El laboratorio altero el chat.'}
    $trainedId=$script:wbLastResult.id
    $labRefresh.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    for($i=0;$i -lt $labModels.Items.Count;$i++){if($labModels.Items[$i].id -eq $trainedId){$labModels.SelectedIndex=$i;break}}
    if($labTrain.Text -ne 'Continuar en GPU'){throw 'No se puede seleccionar modelo para continuar.'}
    $labSteps.SelectedItem='300';$labTrain.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(190)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if($script:wbLastResult.parent_id -ne $trainedId -or $script:wbLastResult.completed_steps -ne 1300){throw ('No continuo desde el modelo seleccionado: '+$wbMessage.Text)}
    $tabs.SelectedTab=$chatPage
    for($i=0;$i -lt $chatModels.Items.Count;$i++){if($chatModels.Items[$i].id -eq $trainedId){$chatModels.SelectedIndex=$i;break}}
    if($research.Enabled -or $contextBudget.Enabled){throw 'Tiny ofrecio herramientas o contexto Qwen.'}
    $prompt.Text='boddy ';$responseBudget.SelectedItem='64';$send.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:active -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if($script:requestMode -ne 'modelo' -or $script:history.Count -ne 0 -or $transcript.Text -notmatch 'continuacion experimental'){throw ('Fallo generacion con Tiny: '+$transcript.Text)}
    $chatModels.SelectedIndex=0
    if(-not $research.Enabled -or -not $contextBudget.Enabled){throw 'No se restauraron controles Qwen.'}
    $script:history.Add(@{role='user';content='Prueba local de conservacion de historial'})
    $script:history.Add(@{role='assistant';content='Registro sintetico sin llamar a Qwen'})
    $chatReload.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if($script:history.Count -ne 2){throw 'Actualizar modelos borro el historial Qwen.'}
    $script:history.Clear()
    'Laboratorio: entrenamiento, continuacion 1300 pasos, selector y generacion Tiny comprobados. '+$trainedId
    $form.Close(); $form.Dispose()
} elseif ($WorkbenchSmokeTest) {
    $form.Opacity=0; $form.Show()
    [Windows.Forms.Application]::DoEvents()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if($tabs.TabPages.Count -ne 8){throw 'Faltan espacios de trabajo.'}
    if($gpuStats.Bottom -gt $chatPage.ClientSize.Height -or $inputPanel.Bottom -gt $chatPage.ClientSize.Height){throw 'Controles de chat fuera del area visible.'}
    $tabs.SelectedTab=$programPage
    $reloadSources.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if($wbFiles.Items.Count -lt 1){throw ('No se cargaron fuentes: '+$wbMessage.Text)}
    $wbFiles.SelectedItem='crates/boddy-inference/src/budget.rs'
    $previewSource.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if($wbPreview.Text -notmatch 'CONTEXT_CAPACITY'){throw 'La vista de fuente no coincide.'}
    $tabs.SelectedTab=$evidencePage
    $refreshTasks.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if($wbTasks.Items.Count -gt 0){
        $wbTasks.SelectedIndex=0; $showTask.PerformClick()
        $deadline=[DateTime]::UtcNow.AddSeconds(20)
        while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
        if($wbRecord.Text -notmatch '"schema"'){throw 'No se cargo el registro seleccionado.'}
    }
    $tabs.SelectedTab=$pcPage
    $refreshGpu.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    if(-not $script:wbLastResult.gpu){throw 'No se recibio el estado de recursos.'}
    'Workbench: ocho pestanas, '+$wbFiles.Items.Count+' fuentes y '+$wbTasks.Items.Count+' registros.'
    $form.Close(); $form.Dispose()
} elseif ($ResearchSmokeTest) {
    $form.Opacity=0
    $form.Show()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    $tabs.SelectedTab=$programPage
    $reloadSources.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while($script:wbProcess -and [DateTime]::UtcNow -lt $deadline){[Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25}
    $wbFiles.SelectedItem='crates/boddy-inference/src/budget.rs'
    if(-not $wbFiles.SelectedItem){throw 'No se pudo seleccionar budget.rs.'}
    $investigateSource.PerformClick()
    $deadline=[DateTime]::UtcNow.AddSeconds(430)
    while ($script:active -and [DateTime]::UtcNow -lt $deadline) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25 }
    if ($script:lastTaskState -notin @('Completed','Stopped') -or $script:lastReply -notmatch 'Registro:') { throw ('No se obtuvo un estado final de la investigacion: '+$transcript.Text) }
    if ($script:lastTaskState -eq 'Completed' -and $script:lastReply -notmatch '\[S[1-3] ') { throw 'Informe aprobado sin citas visibles.' }
    if ($script:lastTaskState -eq 'Stopped' -and $script:lastReply -notmatch 'No se aprobo') { throw 'Tarea detenida presentada como aprobada.' }
    if ($script:history.Count -ne 0) { throw 'La investigacion altero el historial del chat.' }
    'Investigacion en ventana: '+$status.Text
    $form.Close(); $form.Dispose()
} elseif ($SmokeTest) {
    $form.Opacity = 0
    $form.Show()
    $prompt.Text = 'Saluda en espanol en una sola frase.'
    $send.PerformClick()
    $deadline = [DateTime]::UtcNow.AddSeconds(140)
    while ($script:active -and [DateTime]::UtcNow -lt $deadline) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25 }
    if (-not $script:lastReply) { throw ('Fallo chat real: ' + $transcript.Text) }
    if ($script:totalInput -le 0 -or $script:totalOutput -le 0 -or $tokenStats.Text -notmatch 'Ultima:') { throw 'No se mostraron metricas reales de tokens.' }
    $firstInput = $script:totalInput
    $firstOutput = $script:totalOutput
    $prompt.Text = 'Responde solo: listo.'
    $send.PerformClick()
    $deadline = [DateTime]::UtcNow.AddSeconds(140)
    while ($script:active -and [DateTime]::UtcNow -lt $deadline) { [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 25 }
    if ($script:totalInput -le $firstInput -or $script:totalOutput -le $firstOutput -or $script:history.Count -ne 4) { throw 'No se acumularon los tokens de dos turnos.' }
    'Metricas de ventana: ' + $tokenStats.Text + ' | ' + $gpuStats.Text
    if ($PreviewPath) {
        $bitmap = New-Object Drawing.Bitmap($form.Width, $form.Height)
        $form.DrawToBitmap($bitmap, (New-Object Drawing.Rectangle(0,0,$form.Width,$form.Height)))
        $bitmap.Save($PreviewPath, [Drawing.Imaging.ImageFormat]::Png)
        $bitmap.Dispose()
    }
    'Boddy 1.0 chat real: ' + $script:lastReply
    $newChat.PerformClick()
    if ($script:totalInput -ne 0 -or $script:totalOutput -ne 0 -or $script:history.Count -ne 0) { throw 'Nuevo chat no reinicio sus metricas.' }
    'Nuevo chat: historial y contadores reiniciados.'
    $form.Close()
    $form.Dispose()
} else { [void]$form.ShowDialog(); $form.Dispose() }

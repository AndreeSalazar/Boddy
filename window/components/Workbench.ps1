# Espacios de trabajo: solo UI. Las operaciones pasan por comandos del host.
$script:wbProcess = $null
$script:wbCallback = $null
$script:wbLastResult = $null
$wbTimer = New-Object Windows.Forms.Timer
$wbTimer.Interval = 80
function Invoke-WorkbenchCommand([hashtable]$Request, [scriptblock]$OnDone, [ValidateSet('pc','entrenar','modelo','atencion','escuela-rust','biblioteca')][string]$Command='pc') {
    if ($script:wbProcess) { $wbMessage.Text='Hay una accion en curso; espera un momento.'; return }
    $script:wbLastResult=$null
    try {
        $info = New-Object Diagnostics.ProcessStartInfo
        $info.FileName = $binary
        $info.Arguments = $Command
        $info.WorkingDirectory = $projectRoot
        $info.UseShellExecute = $false
        $info.CreateNoWindow = $true
        $info.RedirectStandardInput = $true
        $info.RedirectStandardOutput = $true
        $info.RedirectStandardError = $true
        $info.StandardOutputEncoding = [Text.Encoding]::UTF8
        $info.StandardErrorEncoding = [Text.Encoding]::UTF8
        $script:wbProcess = New-Object Diagnostics.Process
        $script:wbProcess.StartInfo = $info
        [void]$script:wbProcess.Start()
        $script:wbOut = $script:wbProcess.StandardOutput.ReadToEndAsync()
        $script:wbErr = $script:wbProcess.StandardError.ReadToEndAsync()
        $payload = [Text.Encoding]::UTF8.GetBytes(($Request | ConvertTo-Json -Compress))
        $script:wbProcess.StandardInput.BaseStream.Write($payload,0,$payload.Length)
        $script:wbProcess.StandardInput.Close()
        $script:wbCallback = $OnDone
        $script:wbStarted = [DateTime]::UtcNow
        $script:wbCommand = $Command
        $script:wbTimeout = if ($Command -eq 'entrenar' -or $Command -eq 'escuela-rust' -or ($Command -eq 'atencion' -and $Request.action -eq 'train')) { 180 } else { 15 }
        $wbMessage.Text='Consultando: ' + $Request.action
        $wbTimer.Start()
    } catch {
        $wbMessage.Text=$_.Exception.Message
        if ($script:wbProcess) { $script:wbProcess.Dispose(); $script:wbProcess=$null }
    }
}
$wbTimer.Add_Tick({
    if (-not $script:wbProcess) { return }
    if (([DateTime]::UtcNow-$script:wbStarted).TotalSeconds -gt $script:wbTimeout) {
        if (-not $script:wbProcess.HasExited) { $script:wbProcess.Kill() }
        $wbMessage.Text='La accion excedio su tiempo de espera.'
        if ($script:wbCommand -eq 'entrenar') { $labState.Text='Entrenamiento interrumpido por tiempo de espera.' }
        if ($script:wbCommand -eq 'atencion') { $attentionState.Text='Operacion interrumpida por tiempo de espera.' }
        $script:wbProcess.Dispose(); $script:wbProcess=$null; $wbTimer.Stop(); return
    }
    if ($script:wbProcess.HasExited -and $script:wbOut.IsCompleted -and $script:wbErr.IsCompleted) {
        $wbTimer.Stop()
        try {
            if ($script:wbProcess.ExitCode -ne 0) { throw $script:wbErr.Result }
            $script:wbLastResult=$script:wbOut.Result | ConvertFrom-Json
            $wbMessage.Text='Accion finalizada.'
            & $script:wbCallback $script:wbLastResult
        } catch { $wbMessage.Text=$_.Exception.Message; if ($script:wbCommand -eq 'entrenar') { $labState.Text='No se completo el entrenamiento. Ver mensaje inferior.' }; if($script:wbCommand -eq 'atencion'){$attentionState.Text='No se completo la operacion. Ver mensaje inferior.'} }
        finally { $script:wbProcess.Dispose(); $script:wbProcess=$null }
    }
})
function New-WorkbenchButton($Text,$X,$Y,$Width) {
    $button=New-Object Windows.Forms.Button
    $button.Text=$Text; $button.SetBounds($X,$Y,$Width,34)
    $button.FlatStyle='Flat'; $button.FlatAppearance.BorderColor=[Drawing.Color]::DimGray
    return $button
}
function New-WorkbenchPage($Title) {
    $page=New-Object Windows.Forms.TabPage
    $page.Text=$Title; $page.BackColor=$form.BackColor; $page.ForeColor=$form.ForeColor
    return $page
}
$tabs=New-Object Windows.Forms.TabControl
$tabs.Dock='Fill'
$chatPage=New-WorkbenchPage 'Conversar'
$programPage=New-WorkbenchPage 'Programar'
$pcPage=New-WorkbenchPage 'Mi PC'
$evidencePage=New-WorkbenchPage 'Evidencia'
$tabs.TabPages.AddRange(@($chatPage,$programPage,$pcPage,$evidencePage))
$tabs.Size=$form.ClientSize
$oldControls=@($form.Controls | ForEach-Object { $_ })
$script:wbOriginalLayout=@()
$chatPage.SuspendLayout()
foreach ($control in $oldControls) {
    $bounds=$control.Bounds
    $script:wbOriginalLayout+=@{Control=$control; Bounds=$bounds; Anchor=$control.Anchor}
    $chatPage.Controls.Add($control); $control.Bounds=$bounds
}
$chatPage.ResumeLayout()
$form.Controls.Add($tabs)
$wbMessage=New-Object Windows.Forms.Label
$wbMessage.Text='Acciones manuales del propietario | Proyecto: Boddy'
$wbMessage.ForeColor=[Drawing.Color]::Silver
$wbMessage.Dock='Bottom'; $wbMessage.Height=28
$form.Controls.Add($wbMessage)
$wbMessage.BringToFront()
$programTitle=New-Object Windows.Forms.Label
$programTitle.Text='Programar en Boddy'
$programTitle.Font=New-Object Drawing.Font('Segoe UI Semibold',20)
$programTitle.SetBounds(24,20,700,40)
$reloadSources=New-WorkbenchButton 'Fuentes' 24 72 110
$previewSource=New-WorkbenchButton 'Ver fragmento' 144 72 135
$editSource=New-WorkbenchButton 'Abrir editor' 289 72 130
$investigateSource=New-WorkbenchButton 'Investigar seleccion' 429 72 195
$goalMode=New-Object Windows.Forms.ComboBox
$goalMode.DropDownStyle='DropDownList'
$goalMode.Items.AddRange(@('Explicar archivo','Revisar riesgos','Plan de mejora'))
$goalMode.SelectedIndex=0; $goalMode.SetBounds(640,75,210,30)
$wbFiles=New-Object Windows.Forms.ListBox
$wbFiles.SetBounds(24,120,265,570); $wbFiles.Anchor='Top,Bottom,Left'
$wbFiles.BackColor=[Drawing.ColorTranslator]::FromHtml('#282B31'); $wbFiles.ForeColor=$form.ForeColor
$wbFiles.HorizontalScrollbar=$true
$wbPreview=New-Object Windows.Forms.TextBox
$wbPreview.Multiline=$true; $wbPreview.ReadOnly=$true; $wbPreview.ScrollBars='Both'; $wbPreview.WordWrap=$false
$wbPreview.Font=New-Object Drawing.Font('Consolas',10)
$wbPreview.BackColor=[Drawing.ColorTranslator]::FromHtml('#202329'); $wbPreview.ForeColor=$form.ForeColor
$wbPreview.SetBounds(310,120,540,570); $wbPreview.Anchor='Top,Bottom,Left,Right'
$wbPreview.Text='Selecciona una fuente y pulsa Ver fragmento. La lectura no cambia el archivo.'
$programPage.Controls.AddRange(@($programTitle,$reloadSources,$previewSource,$editSource,$investigateSource,$goalMode,$wbFiles,$wbPreview))
$reloadSources.Add_Click({ Invoke-WorkbenchCommand @{action='source_list'} { param($result) $wbFiles.Items.Clear(); foreach($file in $result.files){ [void]$wbFiles.Items.Add([string]$file) }; $wbMessage.Text='Fuentes: '+$wbFiles.Items.Count+' | Listado recortado: '+$result.truncated } })
$previewSource.Add_Click({ if($wbFiles.SelectedItem){ Invoke-WorkbenchCommand @{action='source_preview';path=[string]$wbFiles.SelectedItem} { param($result) $wbPreview.Text=$result.path+"`r`nSHA256: "+$result.sha256+"`r`nFragmento recortado: "+$result.truncated+"`r`n`r`n"+($result.lines -join "`r`n") } } })
$editSource.Add_Click({ if($wbFiles.SelectedItem){ Invoke-WorkbenchCommand @{action='open_editor';path=[string]$wbFiles.SelectedItem} { param($result) $wbMessage.Text='Editor solicitado. PID: '+$result.pid } } })
$investigateSource.Add_Click({
    if($script:active){ $wbMessage.Text='Espera a que termine la tarea activa.'; return }
    if(-not $wbFiles.SelectedItem){ $wbMessage.Text='Selecciona una fuente.'; return }
    $research.Checked=$true
    $objective=switch($goalMode.SelectedIndex){ 1 {'Revisa posibles riesgos y limitaciones de'}; 2 {'Explica el comportamiento y propone un plan de mejora para'}; default {'Explica el proposito y comportamiento de'} }
    $prompt.Text=$objective+' '+[string]$wbFiles.SelectedItem+'. Cita solamente evidencia leida.'
    $tabs.SelectedTab=$chatPage
    $send.PerformClick()
})
$pcTitle=New-Object Windows.Forms.Label
$pcTitle.Text='Tu PC, tus acciones'
$pcTitle.Font=New-Object Drawing.Font('Segoe UI Semibold',20); $pcTitle.SetBounds(24,20,780,45)
$pcScope=New-Object Windows.Forms.Label
$pcScope.Text="Abre herramientas del sistema y espacios del proyecto desde controles identificados.`r`nLas acciones de apertura se registran localmente. El editor se usa manualmente."
$pcScope.SetBounds(24,80,820,65)
$openProject=New-WorkbenchButton 'Abrir proyecto' 24 165 240
$openGraph=New-WorkbenchButton 'SUPER GRAFO' 295 165 240
$openManager=New-WorkbenchButton 'Administrador de tareas' 565 165 270
$openReports=New-WorkbenchButton 'Abrir registros' 24 220 240
$refreshGpu=New-WorkbenchButton 'Actualizar GPU' 295 220 240
$pcMetrics=New-Object Windows.Forms.Label
$pcMetrics.Text='GPU: pulsa Actualizar GPU para consultar una muestra del dispositivo.'
$pcMetrics.Font=New-Object Drawing.Font('Segoe UI',13); $pcMetrics.SetBounds(24,295,800,110)
$pcCapabilities=New-Object Windows.Forms.Label
$pcCapabilities.Text="Disponible: abrir proyecto, editor de una fuente, mapa, registros y Administrador de tareas.`r`nInvestigador: lecturas acotadas y citas verificadas.`r`nPendiente: cambios automaticos, ejecucion aislada y control general de aplicaciones."
$pcCapabilities.SetBounds(24,430,820,150)
$pcPage.Controls.AddRange(@($pcTitle,$pcScope,$openProject,$openGraph,$openManager,$openReports,$refreshGpu,$pcMetrics,$pcCapabilities))
$openProject.Add_Click({ Invoke-WorkbenchCommand @{action='open_project'} {param($r) $wbMessage.Text='Proyecto solicitado. PID: '+$r.pid} })
$openGraph.Add_Click({ Invoke-WorkbenchCommand @{action='open_graph'} {param($r) $wbMessage.Text='SUPER GRAFO solicitado. PID: '+$r.pid} })
$openManager.Add_Click({ Invoke-WorkbenchCommand @{action='open_task_manager'} {param($r) $wbMessage.Text='Administrador de tareas solicitado. PID: '+$r.pid} })
$openReports.Add_Click({ Invoke-WorkbenchCommand @{action='open_reports'} {param($r) $wbMessage.Text='Carpeta de registros solicitada. PID: '+$r.pid} })
$refreshGpu.Add_Click({ Invoke-WorkbenchCommand @{action='resources'} { param($r) if($r.gpu.available){ $pcMetrics.Text='GPU 0 | VRAM total usada: {0:N2} / {1:N2} GiB | Uso: {2}% | Libre: {3:N2} GiB' -f ($r.gpu.used_mib/1024),($r.gpu.total_mib/1024),$r.gpu.utilization_percent,($r.gpu.free_mib/1024) } else { $pcMetrics.Text='Muestra GPU no disponible.' } } })
$evidenceTitle=New-Object Windows.Forms.Label
$evidenceTitle.Text='Evidencia y tareas guardadas'; $evidenceTitle.Font=New-Object Drawing.Font('Segoe UI Semibold',20); $evidenceTitle.SetBounds(24,20,800,45)
$refreshTasks=New-WorkbenchButton 'Actualizar tareas' 24 80 180
$showTask=New-WorkbenchButton 'Ver registro' 220 80 180
$wbTasks=New-Object Windows.Forms.ListBox
$wbTasks.SetBounds(24,135,300,550); $wbTasks.Anchor='Top,Bottom,Left'; $wbTasks.HorizontalScrollbar=$true
$wbTasks.BackColor=$wbFiles.BackColor; $wbTasks.ForeColor=$form.ForeColor
$wbRecord=New-Object Windows.Forms.TextBox
$wbRecord.Multiline=$true; $wbRecord.ReadOnly=$true; $wbRecord.ScrollBars='Both'; $wbRecord.WordWrap=$false
$wbRecord.SetBounds(345,135,505,550); $wbRecord.Anchor='Top,Bottom,Left,Right'; $wbRecord.Font=$wbPreview.Font
$wbRecord.BackColor=$wbPreview.BackColor; $wbRecord.ForeColor=$form.ForeColor
$wbRecord.Text='Las citas comprobadas no garantizan todas las conclusiones del modelo. Revisa el alcance del registro.'
$evidencePage.Controls.AddRange(@($evidenceTitle,$refreshTasks,$showTask,$wbTasks,$wbRecord))
$script:wbTaskIds=@()
$refreshTasks.Add_Click({ Invoke-WorkbenchCommand @{action='task_list'} { param($r) $wbTasks.Items.Clear(); $script:wbTaskIds=@($r.tasks | ForEach-Object { [string]$_.id }); foreach($task in $r.tasks){[void]$wbTasks.Items.Add(($task.state+' | '+$task.objective))}; $wbMessage.Text='Registros mostrados: '+$wbTasks.Items.Count } })
$showTask.Add_Click({ if($wbTasks.SelectedIndex -ge 0){ Invoke-WorkbenchCommand @{action='task_record';id=$script:wbTaskIds[$wbTasks.SelectedIndex]} {param($r) $wbRecord.Text=$r | ConvertTo-Json -Depth 15} } })
$form.Add_Shown({
    $chatPage.SuspendLayout()
    $deltaHeight=$chatPage.ClientSize.Height-850
    $deltaWidth=$chatPage.ClientSize.Width-900
    foreach($item in $script:wbOriginalLayout) {
        $control=$item.Control; $bounds=$item.Bounds; $anchor=$item.Anchor
        $x=$bounds.X; $y=$bounds.Y; $width=$bounds.Width; $height=$bounds.Height
        if($anchor -band [Windows.Forms.AnchorStyles]::Bottom) {
            if($anchor -band [Windows.Forms.AnchorStyles]::Top){$height+=$deltaHeight}else{$y+=$deltaHeight}
        }
        if($anchor -band [Windows.Forms.AnchorStyles]::Right) {
            if($anchor -band [Windows.Forms.AnchorStyles]::Left){$width+=$deltaWidth}else{$x+=$deltaWidth}
        }
        $control.SetBounds($x,$y,[Math]::Max(1,$width),[Math]::Max(1,$height))
        $control.Anchor=$anchor
    }
    $chatPage.ResumeLayout()
})
$form.Add_FormClosed({ $wbTimer.Stop(); $wbTimer.Dispose(); if($script:wbProcess){ if(-not $script:wbProcess.HasExited){$script:wbProcess.Kill()}; $script:wbProcess.Dispose() } })

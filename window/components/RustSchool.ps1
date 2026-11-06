# Solo presentacion. El ranking, la politica y la verificacion pertenecen al host.
$rustPage=New-WorkbenchPage 'Rust'
$tabs.TabPages.Add($rustPage)
$rustTitle=New-Object Windows.Forms.Label
$rustTitle.Text='Escuela Rust | Proponer, verificar, revisar'
$rustTitle.Font=New-Object Drawing.Font('Segoe UI Semibold',19)
$rustTitle.SetBounds(24,20,850,44)
$rustInfo=New-Object Windows.Forms.Label
$rustInfo.Text='Seis ejercicios, nueve expresiones conocidas. El modelo elige; el host comprueba casos. No compila ni ejecuta proyectos y no demuestra programacion general.'
$rustInfo.SetBounds(24,74,850,48);$rustInfo.Anchor='Top,Left,Right'
$rustModels=New-Object Windows.Forms.ComboBox
$rustModels.DropDownStyle='DropDownList';$rustModels.DisplayMember='label'
$rustModels.SetBounds(24,130,620,30);$rustModels.Anchor='Top,Left,Right'
[void]$rustModels.Items.Add([pscustomobject]@{id='';label='Nuevo Attention desde cero (solo para entrenar)'})
$rustModels.SelectedIndex=0
$rustRefresh=New-WorkbenchButton 'Versiones' 660 126 210
$rustRefresh.Anchor='Top,Right'
$rustTrain=New-WorkbenchButton 'Educar Rust: 300 pasos' 24 178 225
$rustAttempts=New-Object Windows.Forms.ComboBox
$rustAttempts.DropDownStyle='DropDownList';$rustAttempts.Items.AddRange(@('1','2','3'));$rustAttempts.SelectedItem='3'
$rustAttempts.SetBounds(268,182,60,30)
$rustRun=New-WorkbenchButton 'Resolver ejercicios' 346 178 210
$rustTasks=New-WorkbenchButton 'Ver ejercicios' 574 178 160
$rustState=New-Object Windows.Forms.Label
$rustState.Text='Intentos por ejercicio: 1 a 3. La busqueda termina al verificar o agotar el cupo.'
$rustState.SetBounds(24,230,850,38);$rustState.Anchor='Top,Left,Right'
$rustOutput=New-Object Windows.Forms.TextBox
$rustOutput.Multiline=$true;$rustOutput.ReadOnly=$true;$rustOutput.ScrollBars='Vertical'
$rustOutput.BackColor=[Drawing.ColorTranslator]::FromHtml('#202329');$rustOutput.ForeColor=$form.ForeColor
$rustOutput.Font=New-Object Drawing.Font('Consolas',10)
$rustOutput.SetBounds(24,284,850,420);$rustOutput.Anchor='Top,Bottom,Left,Right'
$rustOutput.Text="1. Carga una version Attention o entrena desde cero.`r`n2. Educar Rust crea otra version de pesos; conserva la anterior.`r`n3. Resolver inicia un ciclo acotado de eleccion y verificacion.`r`n`r`nSe muestran por separado los aciertos iniciales y los obtenidos por buscar alternativas. Las propuestas aceptadas y su registro se guardan en data/rust-school."
$rustPage.Controls.AddRange(@($rustTitle,$rustInfo,$rustModels,$rustRefresh,$rustTrain,$rustAttempts,$rustRun,$rustTasks,$rustState,$rustOutput))
$rustRefresh.Add_Click({Invoke-WorkbenchCommand @{action='list'} {
    param($result)
    $selected=$rustModels.SelectedItem.id;$rustModels.Items.Clear()
    [void]$rustModels.Items.Add([pscustomobject]@{id='';label='Nuevo Attention desde cero (solo para entrenar)'})
    foreach($item in $result.models){[void]$rustModels.Items.Add([pscustomobject]@{id=$item.id;label=($item.id+' | '+$item.profile)})}
    $rustModels.SelectedIndex=0
    for($i=0;$i -lt $rustModels.Items.Count;$i++){if($rustModels.Items[$i].id -eq $selected){$rustModels.SelectedIndex=$i;break}}
    $rustState.Text='Versiones: '+$result.models.Count+' | Invalidas: '+$result.invalid
} 'atencion'})
$rustTrain.Add_Click({
    if($script:active -or $script:wbProcess){$rustState.Text='Espera a que termine la accion actual.';return}
    $parent=if($rustModels.SelectedItem.id){$rustModels.SelectedItem.id}else{$null}
    $rustState.Text='Entrenando el corpus Rust en GPU...'
    Invoke-WorkbenchCommand @{action='train';steps=300;parent_id=$parent;profile='rust-v1'} {
        param($result)
        $rustState.Text='Version Rust guardada: '+$result.id
        $rustOutput.Text=$result | ConvertTo-Json -Depth 14
        [void]$rustModels.Items.Add([pscustomobject]@{id=$result.id;label=($result.id+' | rust-v1')})
        $rustModels.SelectedIndex=$rustModels.Items.Count-1
    } 'atencion'
})
$rustTasks.Add_Click({Invoke-WorkbenchCommand @{action='tasks'} {param($result) $rustOutput.Text=$result | ConvertTo-Json -Depth 12} 'escuela-rust'})
$rustRun.Add_Click({
    if(-not $rustModels.SelectedItem.id){$rustState.Text='Selecciona o entrena una version Attention.';return}
    if($script:active -or $script:wbProcess){$rustState.Text='Espera a que termine la accion actual.';return}
    $rustState.Text='Eligiendo, verificando y corrigiendo dentro del cupo...'
    Invoke-WorkbenchCommand @{action='run';id=$rustModels.SelectedItem.id;max_attempts=[int]$rustAttempts.SelectedItem} {
        param($result)
        $rustState.Text='Estado: '+$result.state+' | Registro: '+$result.id
        $header="PRIMERA ELECCION: {0}/{2} | TRAS BUSQUEDA: {1}/{2}`r`nSon expresiones conocidas verificadas en casos finitos; no codigo libre.`r`n`r`n" -f $result.first_choice_passed,$result.verified_after_search,$result.task_count
        $code=(@($result.results | Where-Object {$_.source} | ForEach-Object {$_.source}) -join "`r`n")
        $rustOutput.Text=$header+"PROPUESTAS QUE PASARON LOS CASOS:`r`n"+$code+"`r`n`r`nREGISTRO:`r`n"+($result | ConvertTo-Json -Depth 16)
    } 'escuela-rust'
})

# Solo presentacion. Arquitectura, entrenamiento y guardado pertenecen al host Rust.
$labPage=New-WorkbenchPage 'Laboratorio'
$tabs.TabPages.Add($labPage)
$labTitle=New-Object Windows.Forms.Label
$labTitle.Text='Boddy Tiny 1.0 | Pesos propios'
$labTitle.Font=New-Object Drawing.Font('Segoe UI Semibold',20)
$labTitle.SetBounds(24,20,850,44)
$labDescription=New-Object Windows.Forms.Label
$labDescription.Text='Red educativa: 8 caracteres > 32 unidades tanh > 96 salidas. Crea pesos propios o continua una version guardada. Cada etapa conserva su archivo anterior.'
$labDescription.SetBounds(24,74,850,48); $labDescription.Anchor='Top,Left,Right'
$labSteps=New-Object Windows.Forms.ComboBox
$labSteps.DropDownStyle='DropDownList'; $labSteps.Items.AddRange(@('300','1000','2000'))
$labSteps.SelectedIndex=1; $labSteps.SetBounds(24,130,110,30)
$labTrain=New-WorkbenchButton 'Entrenar en GPU' 148 126 190
$labState=New-Object Windows.Forms.Label
$labState.Text='Pasos | Corpus propio ASCII. No descarga modelos ni modifica el chat.'
$labState.SetBounds(352,134,550,36); $labState.Anchor='Top,Left,Right'
$labOutput=New-Object Windows.Forms.TextBox
$labOutput.Multiline=$true; $labOutput.ReadOnly=$true; $labOutput.ScrollBars='Vertical'
$labOutput.SetBounds(24,240,850,440); $labOutput.Anchor='Top,Bottom,Left,Right'
$labOutput.BackColor=[Drawing.ColorTranslator]::FromHtml('#202329'); $labOutput.ForeColor=$form.ForeColor
$labOutput.Font=New-Object Drawing.Font('Consolas',11)
$labOutput.Text="Listo para aprender desde pesos aleatorios.`r`n`r`n27.776 parametros. Cada token es un caracter ASCII.`r`nEl entrenamiento calcula gradientes y actualiza pesos en WGPU.`r`nSe comprueba contra CPU antes de continuar.`r`n`r`nSe guardan checkpoint, metricas y muestra en data/lab.`r`nEsta red permite estudiar aprendizaje; aun no conversa ni programa."
$labPage.Controls.AddRange(@($labTitle,$labDescription,$labSteps,$labTrain,$labState,$labOutput))
$labModels=New-Object Windows.Forms.ComboBox
$labModels.DropDownStyle='DropDownList'; $labModels.DisplayMember='label'
$labModels.SetBounds(24,184,490,30); $labModels.Anchor='Top,Left,Right'
$labRefresh=New-WorkbenchButton 'Modelos guardados' 528 180 165
$labInspect=New-WorkbenchButton 'Ver metricas' 707 180 165
$labRefresh.Anchor='Top,Right'; $labInspect.Anchor='Top,Right'
$labPage.Controls.AddRange(@($labModels,$labRefresh,$labInspect))
$labModels.Items.Add([pscustomobject]@{id='';label='Nuevo Tiny 1.0 desde cero'}) | Out-Null
$labModels.SelectedIndex=0
function Update-SavedModels($result) {
    $selected=if($labModels.SelectedItem){$labModels.SelectedItem.id}else{''}
    $labModels.Items.Clear()
    [void]$labModels.Items.Add([pscustomobject]@{id='';label='Nuevo Tiny 1.0 desde cero'})
    foreach($item in $result.models) {
        $loss=if($null -ne $item.loss.validation_after){'{0:N3}' -f $item.loss.validation_after}else{'n/d'}
        [void]$labModels.Items.Add([pscustomobject]@{id=$item.id;label=($item.id+' | val '+$loss)})
    }
    $labModels.SelectedIndex=0
    for($i=0;$i -lt $labModels.Items.Count;$i++){if($labModels.Items[$i].id -eq $selected){$labModels.SelectedIndex=$i;break}}
    if(Get-Command Update-ChatModels -ErrorAction SilentlyContinue){Update-ChatModels $result}
    $labState.Text='Modelos: '+$result.models.Count+' | Invalidos: '+$result.invalid+' | Lista limitada: '+$result.truncated
}
function Show-LabResult($result) {
    $labOutput.Text=($result | ConvertTo-Json -Depth 14)
    if($result.external_control){
        $summary="VALIDACION: {0:N4} -> {1:N4} | Mejora: {2}`r`nCONTROL NO ENTRENADO: perdida {3:N4} -> {4:N4}; acierto siguiente caracter {5:P1} -> {6:P1}`r`nContinuacion: {7} | Pasos acumulados conocidos: {8}`r`n`r`n" -f $result.loss.validation_before,$result.loss.validation_after,$result.validation_improved,$result.external_control.before.loss,$result.external_control.after.loss,$result.external_control.before.next_character_accuracy,$result.external_control.after.next_character_accuracy,$result.resume_kind,$result.completed_steps
        $labOutput.Text=$summary+$labOutput.Text
    }
}
function Register-LabModel($result) {
    $loss='{0:N3}' -f $result.loss.validation_after
    [void]$labModels.Items.Add([pscustomobject]@{id=$result.id;label=($result.id+' | val '+$loss)})
    $labModels.SelectedIndex=$labModels.Items.Count-1
    if($chatModels){[void]$chatModels.Items.Add([pscustomobject]@{id=$result.id;label=('Tiny 1.0 | '+$result.id+' | Experimental')})}
}
$labModels.Add_SelectedIndexChanged({$labTrain.Text=if($labModels.SelectedItem.id){'Continuar en GPU'}else{'Entrenar en GPU'}})
$labRefresh.Add_Click({Invoke-WorkbenchCommand @{action='list'} {param($result) Update-SavedModels $result} 'modelo'})
$labInspect.Add_Click({
    if(-not $labModels.SelectedItem.id){$labState.Text='Selecciona una version guardada.';return}
    Invoke-WorkbenchCommand @{action='inspect';id=$labModels.SelectedItem.id} {param($result) Show-LabResult $result} 'modelo'
})
$labTrain.Add_Click({
    if ($script:active) { $labState.Text='Espera a que termine la conversacion o investigacion.'; return }
    if ($script:wbProcess) { $labState.Text='Espera a que termine la accion actual.'; return }
    $labState.Text='Entrenando y verificando en GPU...'
    $parent=if($labModels.SelectedItem.id){[string]$labModels.SelectedItem.id}else{$null}
    Invoke-WorkbenchCommand @{action='entrenamiento';steps=[int]$labSteps.SelectedItem;parent_id=$parent} {
        param($result)
        $labState.Text='Experimento guardado: '+$result.id
        Register-LabModel $result
        Show-LabResult $result
    } 'entrenar'
})

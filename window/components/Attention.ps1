# Frontera UI: todos los calculos y archivos pertenecen al host.
$attentionPage=New-WorkbenchPage 'Atencion'
$tabs.TabPages.Add($attentionPage)
$attentionTitle=New-Object Windows.Forms.Label
$attentionTitle.Text='Boddy Attention 0.1 | Experimental'
$attentionTitle.Font=New-Object Drawing.Font('Segoe UI Semibold',20)
$attentionTitle.SetBounds(24,20,850,44)
$attentionInfo=New-Object Windows.Forms.Label
$attentionInfo.Text='UTF-8 | Embeddings 16D | Una cabeza causal | Contexto: 16 bytes | FFN 32 | 10.544 parametros. No es un chat ni sustituye Tiny 1.0.'
$attentionInfo.SetBounds(24,74,850,44);$attentionInfo.Anchor='Top,Left,Right'
$attentionModels=New-Object Windows.Forms.ComboBox
$attentionModels.DropDownStyle='DropDownList';$attentionModels.DisplayMember='label'
$attentionModels.SetBounds(24,124,620,30);$attentionModels.Anchor='Top,Left,Right'
[void]$attentionModels.Items.Add([pscustomobject]@{id='';label='Nuevo Attention 0.1 desde cero'})
$attentionModels.SelectedIndex=0
$attentionRefresh=New-WorkbenchButton 'Versiones' 660 120 210
$attentionRefresh.Anchor='Top,Right'
$attentionSteps=New-Object Windows.Forms.ComboBox
$attentionSteps.DropDownStyle='DropDownList';$attentionSteps.Items.AddRange(@('100','300','1000'));$attentionSteps.SelectedItem='300'
$attentionSteps.SetBounds(24,172,100,30)
$attentionTrain=New-WorkbenchButton 'Entrenar en GPU' 140 168 190
$attentionInspect=New-WorkbenchButton 'Ver metricas' 346 168 150
$attentionState=New-Object Windows.Forms.Label
$attentionState.SetBounds(514,176,350,32);$attentionState.Anchor='Top,Left,Right'
$attentionState.Text='Cinta GPU de referencia; no optimizada.'
$attentionPrefix=New-Object Windows.Forms.TextBox
$attentionPrefix.Text='Boddy: ';$attentionPrefix.SetBounds(24,220,620,30);$attentionPrefix.Anchor='Top,Left,Right'
$attentionGenerate=New-WorkbenchButton 'Continuar texto' 660 216 210
$attentionGenerate.Anchor='Top,Right'
$attentionOutput=New-Object Windows.Forms.TextBox
$attentionOutput.Multiline=$true;$attentionOutput.ReadOnly=$true;$attentionOutput.ScrollBars='Vertical'
$attentionOutput.BackColor=[Drawing.ColorTranslator]::FromHtml('#202329');$attentionOutput.ForeColor=$form.ForeColor
$attentionOutput.Font=New-Object Drawing.Font('Consolas',10)
$attentionOutput.SetBounds(24,274,850,430);$attentionOutput.Anchor='Top,Bottom,Left,Right'
$attentionOutput.Text="Modelo propio con pesos aleatorios. Antes de entrenar se comparan perdida, gradientes y SGD contra CPU.`r`n`r`nLa tokenizacion representa cualquier texto UTF-8. Las muestras usan una mascara de validez UTF-8; no demuestra comprension del idioma.`r`n`r`nLos pesos se guardan en binario junto a un JSON pequeno en data/attention. No se importan pesos Qwen."
$attentionPage.Controls.AddRange(@($attentionTitle,$attentionInfo,$attentionModels,$attentionRefresh,$attentionSteps,$attentionTrain,$attentionInspect,$attentionState,$attentionPrefix,$attentionGenerate,$attentionOutput))
$attentionModels.Add_SelectedIndexChanged({$attentionTrain.Text=if($attentionModels.SelectedItem.id){'Continuar en GPU'}else{'Entrenar en GPU'}})
$attentionRefresh.Add_Click({
    Invoke-WorkbenchCommand @{action='list'} {
        param($result)
        $selected=$attentionModels.SelectedItem.id;$attentionModels.Items.Clear()
        [void]$attentionModels.Items.Add([pscustomobject]@{id='';label='Nuevo Attention 0.1 desde cero'})
        foreach($item in $result.models){[void]$attentionModels.Items.Add([pscustomobject]@{id=$item.id;label=$item.id})}
        $attentionModels.SelectedIndex=0
        for($i=0;$i -lt $attentionModels.Items.Count;$i++){if($attentionModels.Items[$i].id -eq $selected){$attentionModels.SelectedIndex=$i;break}}
        $attentionState.Text='Modelos: '+$result.models.Count+' | Invalidos: '+$result.invalid
    } 'atencion'
})
$attentionTrain.Add_Click({
    if($script:active -or $script:wbProcess){$attentionState.Text='Espera a que termine la accion actual.';return}
    $parent=if($attentionModels.SelectedItem.id){$attentionModels.SelectedItem.id}else{$null}
    $attentionState.Text='Entrenando y verificando...'
    Invoke-WorkbenchCommand @{action='train';steps=[int]$attentionSteps.SelectedItem;parent_id=$parent} {
        param($result)
        $attentionOutput.Text=$result | ConvertTo-Json -Depth 14
        $attentionState.Text='Guardado: '+$result.id
        [void]$attentionModels.Items.Add([pscustomobject]@{id=$result.id;label=$result.id})
        $attentionModels.SelectedIndex=$attentionModels.Items.Count-1
    } 'atencion'
})
$attentionInspect.Add_Click({
    if(-not $attentionModels.SelectedItem.id){$attentionState.Text='Selecciona un modelo guardado.';return}
    Invoke-WorkbenchCommand @{action='inspect';id=$attentionModels.SelectedItem.id} {param($result) $attentionOutput.Text=$result | ConvertTo-Json -Depth 14} 'atencion'
})
$attentionGenerate.Add_Click({
    if(-not $attentionModels.SelectedItem.id){$attentionState.Text='Primero entrena o selecciona una version.';return}
    Invoke-WorkbenchCommand @{action='generate';id=$attentionModels.SelectedItem.id;prefix=$attentionPrefix.Text;max_tokens=64} {param($result) $attentionOutput.Text='Continuacion experimental (CPU, UTF-8):'+"`r`n"+$result.content;$attentionState.Text='Texto generado; no es una respuesta a instrucciones.'} 'atencion'
})

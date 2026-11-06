# Eleccion de backend visible; el host valida y genera. Tiny no recibe historial ni herramientas.
$subtitle.Text='Modelo activo'
$subtitle.SetBounds(32,76,110,28)
$chatModels=New-Object Windows.Forms.ComboBox
$chatModels.DropDownStyle='DropDownList'; $chatModels.DisplayMember='label'
$chatModels.SetBounds(154,74,550,30); $chatModels.Anchor='Top,Left,Right'
$chatReload=New-WorkbenchButton 'Actualizar' 716 72 150
$chatReload.Anchor='Top,Right'
$chatPage.Controls.AddRange(@($chatModels,$chatReload))
$chatModels.Items.Add([pscustomobject]@{id='qwen-local';label='Qwen2.5-Coder 7B | Chat e investigacion'}) | Out-Null
$chatModels.SelectedIndex=0
function Update-ModelControls {
    $tiny=$chatModels.SelectedItem -and $chatModels.SelectedItem.id -ne 'qwen-local'
    $research.Enabled=-not $tiny -and -not $script:active
    if($tiny){$research.Checked=$false}
    $contextBudget.Enabled=-not $tiny -and -not $script:active -and -not $research.Checked
    $useGraph.Enabled=$contextBudget.Enabled
    $responseBudget.Enabled=-not $script:active -and -not $research.Checked
    $chatModels.Enabled=-not $script:active
    $chatReload.Enabled=-not $script:active
    if($tiny){
        if([int]$responseBudget.SelectedItem -gt 256){$responseBudget.SelectedItem='256'}
        $responseBudget.Items.Remove('512');$responseBudget.Items.Remove('1024')
        $hint.Text='Tiny experimental: continua los ultimos 8 caracteres ASCII. Sin historial ni herramientas; no responde instrucciones.'
        $status.Text='Modelo propio: '+$chatModels.SelectedItem.id+' | Generacion CPU de referencia'
    } elseif(-not $script:active){
        if(-not $responseBudget.Items.Contains('512')){[void]$responseBudget.Items.Add('512');[void]$responseBudget.Items.Add('1024')}
        $hint.Text=if($research.Checked){'Lectura: hasta 3 archivos, 4 llamadas al modelo y 1 correccion.'}else{'Qwen: chat local. El presupuesto recorta turnos antiguos; el mapa aporta cifras.'}
    }
}
function Update-ChatModels($result) {
    $selected=$chatModels.SelectedItem.id
    $script:refreshingModels=$true
    $chatModels.Items.Clear()
    [void]$chatModels.Items.Add([pscustomobject]@{id='qwen-local';label='Qwen2.5-Coder 7B | Chat e investigacion'})
    foreach($item in $result.models){[void]$chatModels.Items.Add([pscustomobject]@{id=$item.id;label=('Tiny 1.0 | '+$item.id+' | Experimental')})}
    $chatModels.SelectedIndex=0
    for($i=0;$i -lt $chatModels.Items.Count;$i++){if($chatModels.Items[$i].id -eq $selected){$chatModels.SelectedIndex=$i;break}}
    $script:refreshingModels=$false
    Update-ModelControls
}
$chatModels.Add_SelectedIndexChanged({
    if($script:active -or $script:refreshingModels){return}
    $script:history.Clear();$script:totalInput=0L;$script:totalOutput=0L;$script:totalCached=0L
    $tokenStats.Text='Contadores reiniciados al cambiar el modelo.'
    if($chatModels.SelectedItem.id -eq 'qwen-local'){$status.Text='Modelo local: Qwen2.5-Coder 7B | Chat e investigacion'}
    Update-ModelControls
})
$chatReload.Add_Click({Invoke-WorkbenchCommand @{action='list'} {param($result) Update-SavedModels $result} 'modelo'})
$form.Add_Shown({$chatReload.PerformClick()})

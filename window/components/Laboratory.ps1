# Solo presentacion. Arquitectura, entrenamiento y guardado pertenecen al host Rust.
$labPage=New-WorkbenchPage 'Laboratorio'
$tabs.TabPages.Add($labPage)
$labTitle=New-Object Windows.Forms.Label
$labTitle.Text='Boddy Tiny 1.0 | Pesos propios'
$labTitle.Font=New-Object Drawing.Font('Segoe UI Semibold',20)
$labTitle.SetBounds(24,20,850,44)
$labDescription=New-Object Windows.Forms.Label
$labDescription.Text='Red educativa: 8 caracteres > 32 unidades tanh > 96 salidas. Sin pesos de Qwen. Cada experimento empieza desde cero con semilla 1337.'
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
$labOutput.SetBounds(24,186,850,480); $labOutput.Anchor='Top,Bottom,Left,Right'
$labOutput.BackColor=[Drawing.ColorTranslator]::FromHtml('#202329'); $labOutput.ForeColor=$form.ForeColor
$labOutput.Font=New-Object Drawing.Font('Consolas',11)
$labOutput.Text="Listo para aprender desde pesos aleatorios.`r`n`r`n27.776 parametros. Cada token es un caracter ASCII.`r`nEl entrenamiento calcula gradientes y actualiza pesos en WGPU.`r`nSe comprueba contra CPU antes de continuar.`r`n`r`nSe guardan checkpoint, metricas y muestra en data/lab.`r`nEsta red permite estudiar aprendizaje; aun no conversa ni programa."
$labPage.Controls.AddRange(@($labTitle,$labDescription,$labSteps,$labTrain,$labState,$labOutput))
$labTrain.Add_Click({
    if ($script:active) { $labState.Text='Espera a que termine la conversacion o investigacion.'; return }
    if ($script:wbProcess) { $labState.Text='Espera a que termine la accion actual.'; return }
    $labState.Text='Entrenando y verificando en GPU...'
    Invoke-WorkbenchCommand @{action='entrenamiento desde cero';steps=[int]$labSteps.SelectedItem} {
        param($result)
        $labState.Text='Experimento guardado: '+$result.id
        $labOutput.Text=("{0} | {1}`r`n{2}`r`n`r`nParametros: {3:N0} | Pasos: {4} | Tiempo: {5:N2} s`r`nTokens objetivo procesados: {6:N0}`r`nTokens de contexto por lote: {7} | Contexto: {8}`r`n`r`nPerdida (menor es mejor)`r`nEntrenamiento: {9:N4} -> {10:N4}`r`nValidacion:    {11:N4} -> {12:N4}`r`n`r`nBuffers GPU: {13:N0} bytes (excluye driver y memoria adicional)`r`nPesos f32: {14:N0} bytes`r`nError maximo gradiente CPU/GPU: {15:E3}`r`nCheckpoint recargado: {16}`r`n`r`nArchivo: {17}`r`nMetricas: {18}`r`n`r`nMUESTRA (generada con referencia CPU):`r`n{19}`r`n`r`nLIMITES: ASCII, contexto corto, corpus pequeno. No mide inteligencia general." -f $result.model,$result.backend,$result.device,$result.parameters,$result.steps,($result.elapsed_ms/1000),$result.training_target_tokens,$result.batch_tokens,$result.context_tokens,$result.loss.train_before,$result.loss.train_after,$result.loss.validation_before,$result.loss.validation_after,$result.tensor_buffer_bytes,$result.weight_f32_bytes,$result.parity.gradient_max_abs,$result.checkpoint_roundtrip,$result.checkpoint_path,$result.metrics_path,$result.sample)
    } 'entrenar'
})

# Solo UI; catalogo, lectura, recuerdos y exportacion pertenecen al host Rust.
$libraryPage=New-WorkbenchPage 'Biblioteca';$tabs.TabPages.Add($libraryPage)
$libraryTitle=New-Object Windows.Forms.Label
$libraryTitle.Text='Biblioteca | Localizar, leer, recordar';$libraryTitle.Font=New-Object Drawing.Font('Segoe UI Semibold',20)
$libraryTitle.SetBounds(24,18,850,46)
$libraryInfo=New-Object Windows.Forms.Label
$libraryInfo.Text='La Forja / Babel / Agora / Atelier / Bitacora. Memoria documental, no entrenamiento. Los modelos propios aun tienen contexto muy corto.'
$libraryInfo.SetBounds(24,74,850,46);$libraryInfo.Anchor='Top,Left,Right'
$libraryCategory=New-Object Windows.Forms.ComboBox
$libraryCategory.DropDownStyle='DropDownList';$libraryCategory.DisplayMember='label'
[void]$libraryCategory.Items.Add([pscustomobject]@{id='';label='Todas'})
$libraryCategory.SelectedIndex=0;$libraryCategory.SetBounds(24,130,160,30)
$libraryQuery=New-Object Windows.Forms.TextBox
$libraryQuery.Text='rust prestamos';$libraryQuery.SetBounds(196,130,380,30);$libraryQuery.Anchor='Top,Left,Right'
$librarySearch=New-WorkbenchButton 'Buscar fichas' 592 126 135;$librarySearch.Anchor='Top,Right'
$libraryLanguages=New-WorkbenchButton 'Idiomas IANA' 741 126 135;$libraryLanguages.Anchor='Top,Right'
$libraryBudget=New-Object Windows.Forms.ComboBox
$libraryBudget.DropDownStyle='DropDownList';$libraryBudget.Items.AddRange(@('256','512','1024','2048','4096'));$libraryBudget.SelectedItem='1024';$libraryBudget.SetBounds(24,182,80,30)
$libraryContext=New-WorkbenchButton 'Contexto' 116 178 100
$libraryRead=New-WorkbenchButton 'Ver fragmento' 228 178 120
$libraryRemember=New-WorkbenchButton 'Recordar IDs' 360 178 120
$libraryExport=New-WorkbenchButton 'Exportar MD/JSON' 492 178 160
$libraryCards=New-Object Windows.Forms.ListBox
$libraryCards.DisplayMember='label';$libraryCards.SetBounds(24,236,245,455);$libraryCards.Anchor='Top,Bottom,Left'
$libraryCards.BackColor=[Drawing.ColorTranslator]::FromHtml('#282B31');$libraryCards.ForeColor=$form.ForeColor
$libraryOutput=New-Object Windows.Forms.TextBox
$libraryOutput.Multiline=$true;$libraryOutput.ReadOnly=$true;$libraryOutput.ScrollBars='Vertical';$libraryOutput.Font=New-Object Drawing.Font('Consolas',10)
$libraryOutput.BackColor=[Drawing.ColorTranslator]::FromHtml('#202329');$libraryOutput.ForeColor=$form.ForeColor
$libraryOutput.SetBounds(285,236,590,455);$libraryOutput.Anchor='Top,Bottom,Left,Right'
$libraryOutput.Text="Buscar solo consulta el registro. Contexto lee hasta tres documentos y selecciona fragmentos con fuente y SHA256.`r`n`r`nEl presupuesto es de bytes UTF-8, no tokens del modelo. Recordar guarda referencias; releer detecta cambios.`r`n`r`nBabel cataloga idiomas; no implica cursos completos ni dominio del modelo."
$libraryMemory=New-Object Windows.Forms.ComboBox
$libraryMemory.DropDownStyle='DropDownList';$libraryMemory.DisplayMember='label';$libraryMemory.SetBounds(24,710,500,30);$libraryMemory.Anchor='Bottom,Left,Right'
$libraryLoadMemory=New-WorkbenchButton 'Recuerdos' 540 706 130;$libraryLoadMemory.Anchor='Bottom,Right'
$libraryRecall=New-WorkbenchButton 'Releer' 686 706 130;$libraryRecall.Anchor='Bottom,Right'
$libraryState=New-Object Windows.Forms.Label
$libraryState.Text='Presupuesto: bytes UTF-8 | Notas propias iniciales; conocimiento parcial.'
$libraryState.SetBounds(24,752,850,28);$libraryState.Anchor='Bottom,Left,Right'
$libraryPage.Controls.AddRange(@($libraryTitle,$libraryInfo,$libraryCategory,$libraryQuery,$librarySearch,$libraryLanguages,$libraryBudget,$libraryContext,$libraryRead,$libraryRemember,$libraryExport,$libraryCards,$libraryOutput,$libraryMemory,$libraryLoadMemory,$libraryRecall,$libraryState))
$libraryCategories=New-WorkbenchButton 'Categorias' 730 20 146
$libraryCategories.Anchor='Top,Right';$libraryPage.Controls.Add($libraryCategories)
$libraryCategories.Add_Click({Invoke-WorkbenchCommand @{action='categories'} {param($r) $selected=$libraryCategory.SelectedItem.id;$libraryCategory.Items.Clear();[void]$libraryCategory.Items.Add([pscustomobject]@{id='';label='Todas'});foreach($c in $r.categories){[void]$libraryCategory.Items.Add([pscustomobject]@{id=$c.id;label=$c.name})};$libraryCategory.SelectedIndex=0;for($i=0;$i -lt $libraryCategory.Items.Count;$i++){if($libraryCategory.Items[$i].id -eq $selected){$libraryCategory.SelectedIndex=$i;break}};$libraryState.Text='Fichas registradas: '+$r.cards;Show-LibraryPacket $r} 'biblioteca'})
function New-LibraryRequest($Action){$req=@{action=$Action;query=$libraryQuery.Text;budget_bytes=[int]$libraryBudget.SelectedItem};if($libraryCategory.SelectedItem.id){$req.category=$libraryCategory.SelectedItem.id};return $req}
function Show-LibraryPacket($result){$libraryOutput.Text=$result | ConvertTo-Json -Depth 16;if($null -ne $result.context_bytes){$libraryState.Text='Contexto: '+$result.context_bytes+'/'+$result.budget_bytes+' bytes | Documentos: '+$result.documents_read+' | Tokens: no medidos';$libraryOutput.Text=$result.context+"`r`n`r`nREFERENCIAS Y MEDIDAS:`r`n"+$libraryOutput.Text}}
$librarySearch.Add_Click({Invoke-WorkbenchCommand (New-LibraryRequest 'search') {param($r) $libraryCards.Items.Clear();foreach($c in $r.cards){[void]$libraryCards.Items.Add([pscustomobject]@{id=$c.id;label=$c.title})};if($libraryCards.Items.Count){$libraryCards.SelectedIndex=0};$libraryState.Text='Fichas: '+$r.matches+' | Documentos abiertos: '+$r.documents_read;Show-LibraryPacket $r} 'biblioteca'})
$libraryContext.Add_Click({Invoke-WorkbenchCommand (New-LibraryRequest 'context') {param($r) Show-LibraryPacket $r} 'biblioteca'})
$libraryRead.Add_Click({if(-not $libraryCards.SelectedItem){$libraryState.Text='Selecciona una ficha.';return};$req=New-LibraryRequest 'read';$req.id=$libraryCards.SelectedItem.id;Invoke-WorkbenchCommand $req {param($r) Show-LibraryPacket $r} 'biblioteca'})
$libraryRemember.Add_Click({Invoke-WorkbenchCommand (New-LibraryRequest 'remember') {param($r) [void]$libraryMemory.Items.Add([pscustomobject]@{id=$r.memory_id;label=$r.memory_id});$libraryMemory.SelectedIndex=$libraryMemory.Items.Count-1;$libraryState.Text='Recuerdo guardado: '+$r.bytes+' bytes';Show-LibraryPacket $r} 'biblioteca'})
$libraryLoadMemory.Add_Click({Invoke-WorkbenchCommand @{action='memories'} {param($r) $libraryMemory.Items.Clear();foreach($m in $r.memories){[void]$libraryMemory.Items.Add([pscustomobject]@{id=$m.memory_id;label=($m.memory_id+' | '+$m.query)})};if($libraryMemory.Items.Count){$libraryMemory.SelectedIndex=0};$libraryState.Text='Recuerdos: '+$r.memories.Count+' | Invalidos: '+$r.invalid} 'biblioteca'})
$libraryRecall.Add_Click({if(-not $libraryMemory.SelectedItem){$libraryState.Text='Selecciona un recuerdo.';return};$req=New-LibraryRequest 'recall';$req.memory_id=$libraryMemory.SelectedItem.id;Invoke-WorkbenchCommand $req {param($r) Show-LibraryPacket $r} 'biblioteca'})
$libraryExport.Add_Click({Invoke-WorkbenchCommand (New-LibraryRequest 'export') {param($r) Show-LibraryPacket $r;$libraryState.Text='Exportados JSON y Markdown: '+$r.json_path} 'biblioteca'})
$libraryLanguages.Add_Click({Invoke-WorkbenchCommand @{action='languages';query=$libraryQuery.Text} {param($r) Show-LibraryPacket $r;$libraryState.Text='Idiomas: '+$r.records+' | Coincidencias: '+$r.matched+' | Cursos pendientes'} 'biblioteca'})

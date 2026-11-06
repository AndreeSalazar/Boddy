param([string]$ParentId,[string]$EvidencePath)
$ErrorActionPreference='Stop'
$OutputEncoding=[Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$taskBinary=Join-Path $taskRoot 'target\debug\boddy-cli.exe'
function Invoke-School([string]$Command,[hashtable]$Request){
    $raw=($Request | ConvertTo-Json -Compress) | & $taskBinary $Command
    if($LASTEXITCODE -ne 0){throw 'Fallo el comando de Escuela Rust.'}
    return ($raw -join "`n" | ConvertFrom-Json)
}
if(-not $ParentId){$initial=Invoke-School 'atencion' @{action='train';steps=100;profile='general-v1'};$ParentId=$initial.id}
if($ParentId -notmatch '^attn-[0-9]{1,30}$'){throw 'Identificador padre invalido.'}
$parentPath=Join-Path $taskRoot ('data/attention/'+$ParentId+'/checkpoint.json')
$parentHash=(Get-FileHash -LiteralPath $parentPath -Algorithm SHA256).Hash
$before=Invoke-School 'escuela-rust' @{action='run';id=$ParentId;max_attempts=3}
$trained=Invoke-School 'atencion' @{action='train';steps=300;parent_id=$ParentId;profile='rust-v1'}
if($trained.training_profile -ne 'rust-v1' -or $trained.resume_kind -ne 'transfer_profile'){throw 'No se transfirieron los pesos al perfil Rust.'}
if((Get-FileHash -LiteralPath $parentPath -Algorithm SHA256).Hash -ne $parentHash){throw 'Se modifico el checkpoint padre.'}
$one=Invoke-School 'escuela-rust' @{action='run';id=$trained.id;max_attempts=1}
$after=Invoke-School 'escuela-rust' @{action='run';id=$trained.id;max_attempts=3}
if($one.first_choice_passed -ne $after.first_choice_passed -or $one.verified_after_search -ne $one.first_choice_passed -or $after.verified_after_search -lt $one.verified_after_search){throw 'Metricas de primera eleccion y busqueda incoherentes.'}
foreach($result in $after.results){
    if($result.attempts.Count -gt 3){throw 'Se excedio el presupuesto.'}
    if($result.state -eq 'Completed' -and ($result.attempts[-1].cases_passed -ne 5 -or -not $result.source)){throw 'Se aprobo una propuesta sin verificacion.'}
}
if($after.policy.execute_code -or $after.policy.network){throw 'La escuela habilito ejecucion o red.'}
$evidence=[ordered]@{schema=1;test='rust-school-v1';parent_model=$ParentId;rust_model=$trained.id;parent_unchanged=$true;training_profile=$trained.training_profile;resume_kind=$trained.resume_kind;loss=$trained.loss;before=@{first_choice=$before.first_choice_passed;search=$before.verified_after_search;tasks=$before.task_count;journal=$before.journal_path};after=@{first_choice=$after.first_choice_passed;search=$after.verified_after_search;tasks=$after.task_count;journal=$after.journal_path};single_attempt_solved=$one.verified_after_search;state=$after.state;source_path=$after.source_path;policy=$after.policy;limitations=$after.limitations}
if($EvidencePath){[IO.File]::WriteAllText($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($EvidencePath),($evidence | ConvertTo-Json -Depth 12),[Text.UTF8Encoding]::new($false))}
$evidence | ConvertTo-Json -Depth 12

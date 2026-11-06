param([string]$EvidencePath)
$ErrorActionPreference='Stop'
$taskRoot=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$taskBinary=Join-Path $taskRoot 'target\debug\boddy-cli.exe'
function Invoke-Tiny([string]$Command,[hashtable]$Request) {
    $raw=($Request | ConvertTo-Json -Compress) | & $taskBinary $Command
    if($LASTEXITCODE -ne 0){throw 'Fallo el comando Tiny.'}
    return ($raw -join "`n" | ConvertFrom-Json)
}
$full=Invoke-Tiny 'entrenar' @{steps=600}
$parent=Invoke-Tiny 'entrenar' @{steps=300}
$parentPath=Join-Path $taskRoot $parent.checkpoint_path
$beforeHash=(Get-FileHash -LiteralPath $parentPath -Algorithm SHA256).Hash
$child=Invoke-Tiny 'entrenar' @{steps=300;parent_id=$parent.id}
$afterHash=(Get-FileHash -LiteralPath $parentPath -Algorithm SHA256).Hash
if($beforeHash -ne $afterHash){throw 'Se modifico el checkpoint padre.'}
$fullCheckpoint=Get-Content -LiteralPath (Join-Path $taskRoot $full.checkpoint_path) -Raw | ConvertFrom-Json
$childCheckpoint=Get-Content -LiteralPath (Join-Path $taskRoot $child.checkpoint_path) -Raw | ConvertFrom-Json
$maxError=0.0
if($fullCheckpoint.schema -ne 3 -or $childCheckpoint.schema -ne 3){throw 'Se esperaba formato binario v3.'}
$fullBytes=[IO.File]::ReadAllBytes((Join-Path $taskRoot $full.storage.weights_path))
$childBytes=[IO.File]::ReadAllBytes((Join-Path $taskRoot $child.storage.weights_path))
if($fullBytes.Length -ne 111104 -or $childBytes.Length -ne 111104){throw 'Cantidad de pesos incorrecta.'}
for($i=0;$i -lt 27776;$i++){
    $maxError=[Math]::Max($maxError,[Math]::Abs([BitConverter]::ToSingle($fullBytes,$i*4)-[BitConverter]::ToSingle($childBytes,$i*4)))
}
if($maxError -gt 0.000001 -or $child.completed_steps -ne 600 -or $child.resume_kind -ne 'full_state' -or $fullCheckpoint.sampler_state -ne $childCheckpoint.sampler_state){throw ('Continuacion no reproducible: '+$maxError)}
if(-not $child.storage.blob_reused -or $child.storage.new_weight_bytes -ne 0 -or $child.storage.weights_path -ne $full.storage.weights_path -or $child.storage.manifest_bytes -gt 2048){throw 'No se reutilizaron pesos identicos o el JSON crecio demasiado.'}
$generated=Invoke-Tiny 'modelo' @{action='generate';id=$child.id;prefix='boddy ';max_tokens=32}
if($generated.backend -ne 'CPU reference' -or $generated.completion_tokens -ne 32 -or $generated.content.Length -ne 40){throw 'Generacion Tiny incoherente.'}
$registry=Invoke-Tiny 'modelo' @{action='list'}
if($child.id -notin @($registry.models | ForEach-Object {$_.id})){throw 'Modelo continuado ausente del registro.'}
$evidence=[ordered]@{schema=2;test='tiny-continuation-storage-v3';device=$child.device;full_id=$full.id;parent_id=$parent.id;child_id=$child.id;full_steps=600;split_steps=@(300,300);maximum_weight_absolute_error=$maxError;sampler_equal=($fullCheckpoint.sampler_state -eq $childCheckpoint.sampler_state);parent_sha256_unchanged=($beforeHash -eq $afterHash);generated_tokens=$generated.completion_tokens;storage=$child.storage;loss=$child.loss;external_control=$child.external_control}
if($EvidencePath){[IO.File]::WriteAllText($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($EvidencePath),($evidence | ConvertTo-Json -Depth 10),[Text.UTF8Encoding]::new($false))}
$evidence | ConvertTo-Json -Depth 10

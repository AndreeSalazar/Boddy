param([string]$EvidencePath)
$ErrorActionPreference='Stop'
$OutputEncoding=[Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$taskBinary=Join-Path $taskRoot 'target\debug\boddy-cli.exe'
function Invoke-Attention([hashtable]$Request){
    $raw=($Request | ConvertTo-Json -Compress) | & $taskBinary atencion
    if($LASTEXITCODE -ne 0){throw 'Fallo el comando Attention.'}
    return ($raw -join "`n" | ConvertFrom-Json)
}
$full=Invoke-Attention @{action='train';steps=200}
$parent=Invoke-Attention @{action='train';steps=100}
$parentHash=(Get-FileHash -LiteralPath (Join-Path $taskRoot $parent.checkpoint_path) -Algorithm SHA256).Hash
$child=Invoke-Attention @{action='train';steps=100;parent_id=$parent.id}
$fullManifest=Get-Content -LiteralPath (Join-Path $taskRoot $full.checkpoint_path) -Raw | ConvertFrom-Json
$childManifest=Get-Content -LiteralPath (Join-Path $taskRoot $child.checkpoint_path) -Raw | ConvertFrom-Json
if($fullManifest.weights_sha256 -ne $childManifest.weights_sha256 -or $fullManifest.sampler_state -ne $childManifest.sampler_state -or $child.completed_steps -ne 200){throw 'La continuacion Attention no reprodujo el entrenamiento completo.'}
if((Get-FileHash -LiteralPath (Join-Path $taskRoot $parent.checkpoint_path) -Algorithm SHA256).Hash -ne $parentHash){throw 'Se modifico el padre Attention.'}
$prefix=([char]0x00bf)+'Qu'+([char]0x00e9)+' tal? '+[char]::ConvertFromUtf32(0x1f9e0)
$generated=Invoke-Attention @{action='generate';id=$child.id;prefix=$prefix;max_tokens=32}
if(-not $generated.content.StartsWith($prefix)){throw 'No se preservo el prefijo Unicode.'}
$evidence=[ordered]@{schema=1;test='attention-continuation-v1';device=$full.device;full_id=$full.id;parent_id=$parent.id;child_id=$child.id;full_steps=200;split_steps=@(100,100);weights_sha256_equal=$true;sampler_equal=$true;parent_unchanged=$true;unicode_prefix_preserved=$true;parity=$full.parity;loss=$full.loss;storage=$full.storage;limitations=$full.limitations}
if($EvidencePath){[IO.File]::WriteAllText($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($EvidencePath),($evidence | ConvertTo-Json -Depth 12),[Text.UTF8Encoding]::new($false))}
$evidence | ConvertTo-Json -Depth 12

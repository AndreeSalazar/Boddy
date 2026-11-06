param([ValidateSet(0,1)][int]$RepairAttempts=1)
$ErrorActionPreference='Stop'
$projectRoot=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$binary=Join-Path $projectRoot 'target\debug\boddy-cli.exe'
$suite=Get-Content -LiteralPath (Join-Path $projectRoot 'evals\research-v1.json') -Raw | ConvertFrom-Json
$lock=Get-Content -LiteralPath (Join-Path $projectRoot 'models.lock.json') -Raw | ConvertFrom-Json
$results=@()
foreach($case in $suite.cases) {
    $request=@{objective=$case.objective;repair_attempts=$RepairAttempts}|ConvertTo-Json -Compress
    $raw=$request | & $binary investigar
    if($LASTEXITCODE -ne 0){throw ('Fallo del cliente en '+$case.id)}
    $reply=$raw|ConvertFrom-Json
    if($reply.task.model_calls -gt (3+$RepairAttempts) -or $reply.task.tool_calls -gt 6 -or $reply.task.repairs_used -gt $RepairAttempts){throw 'El agente excedio los limites.'}
    if($reply.task.semantic_review -ne 'pending'){throw 'Revision semantica indebidamente aprobada.'}
    $expected=@($reply.task.sources|Where-Object {$_.path -eq $case.expected_source}).Count -gt 0
    $results+=@{id=$case.id;objective=$case.objective;state=$reply.task.state;expected_source_read=$expected;citation_case_pass=($reply.task.state -eq 'Completed' -and $expected);model_calls=$reply.task.model_calls;repairs_used=$reply.task.repairs_used;prompt_tokens=$reply.usage.prompt_tokens;completion_tokens=$reply.usage.completion_tokens;error=$reply.task.error;journal_path=$reply.journal_path;semantic_correctness='not_evaluated'}
}
$passed=@($results|Where-Object {$_.citation_case_pass}).Count
$output=@{suite_version=$suite.version;metric=$suite.metric;repair_limit=$RepairAttempts;model_sha256=$lock.model_sha256;runtime=$lock.runtime;passed=$passed;total=$results.Count;cases=$results}
$folder=Join-Path $projectRoot 'data\evaluations'
[IO.Directory]::CreateDirectory($folder)|Out-Null
$file=Join-Path $folder ('research-v1-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()+'.json')
$output|ConvertTo-Json -Depth 10|Set-Content -LiteralPath $file -Encoding utf8
$output|ConvertTo-Json -Depth 10
'Guardado: '+$file

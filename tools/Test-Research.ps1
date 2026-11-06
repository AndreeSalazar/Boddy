$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$binary = Join-Path $projectRoot 'target\debug\boddy-cli.exe'
$request = @{objective='Explica como budget.rs limita el contexto de tokens de Boddy.'} | ConvertTo-Json -Compress
$raw = $request | & $binary investigar
if ($LASTEXITCODE -ne 0) { throw 'Fallo del cliente de investigacion.' }
$reply = $raw | ConvertFrom-Json
if ($reply.task.state -ne 'Completed') { throw ('La tarea no completo sus citas: '+$reply.content) }
if ($reply.task.model_calls -gt 4 -or $reply.task.tool_calls -gt 6 -or $reply.task.repairs_used -gt 1) { throw 'Se excedieron los cupos de investigacion.' }
if ($reply.task.semantic_review -ne 'pending') { throw 'Se aprobo indebidamente la semantica del modelo.' }
$journal = Get-Content -LiteralPath (Join-Path $projectRoot $reply.journal_path) -Raw | ConvertFrom-Json
if ($journal.id -ne $reply.task.id -or $journal.state -ne 'Completed') { throw 'Registro persistente no coincide con el resultado.' }
if ($journal.sources.Count -lt 1 -or $journal.report.findings.Count -lt 1) { throw 'Falta evidencia.' }
$reply | Select-Object content,journal_path,@{Name='state';Expression={$_.task.state}} | ConvertTo-Json -Depth 5

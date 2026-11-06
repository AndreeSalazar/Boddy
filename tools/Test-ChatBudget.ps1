$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$binary = Join-Path $projectRoot 'target\debug\boddy-cli.exe'
$history = @()
for ($i = 0; $i -lt 6; $i++) {
    $history += @{role='user';content=('Detalle antiguo sin relacion con la tarea. ' * 18)}
    $history += @{role='assistant';content=('Confirmacion de detalles antiguos. ' * 10)}
}
$history += @{role='user';content='Responde solo con el nombre: FaroAzul.'}
$request = @{messages=$history;context_tokens=512;max_tokens=64} | ConvertTo-Json -Depth 5 -Compress
$raw = $request | & $binary chat
if ($LASTEXITCODE -ne 0) { throw ('Fallo del cliente: ' + $raw) }
$reply = $raw | ConvertFrom-Json
if ($reply.budget.dropped_messages -le 0) { throw 'No se recorto historial.' }
if (($reply.budget.prompt_after+64+64) -gt 512) { throw 'Presupuesto excedido.' }
if ($reply.usage.prompt_tokens -ne $reply.budget.prompt_after) { throw 'Conteo previo no coincide con uso del motor.' }
if ($reply.context_messages[-1].content -ne $history[-1].content) { throw 'La pregunta actual se modifico.' }
$reply | Select-Object content,budget,usage,gpu | ConvertTo-Json -Depth 8

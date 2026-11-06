$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$pidFile = Join-Path $projectRoot 'runtime\server.pid'
if (-not (Test-Path -LiteralPath $pidFile)) { 'No hay motor registrado.'; return }
$modelProcessId = [int](Get-Content -LiteralPath $pidFile -Raw)
$process = Get-Process -Id $modelProcessId -ErrorAction SilentlyContinue
$expected = [IO.Path]::GetFullPath((Join-Path $projectRoot 'runtime\llama\llama-server.exe'))
if ($process -and $process.Path -eq $expected) { Stop-Process -Id $modelProcessId; 'Motor detenido; memoria GPU liberada.' }
elseif ($process) { throw 'El PID pertenece a otro programa; no se ha detenido.' }
else { 'El motor ya estaba detenido.' }

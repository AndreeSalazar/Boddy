$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$server = Join-Path $projectRoot 'runtime\llama\llama-server.exe'
$model = Join-Path $projectRoot 'models\qwen2.5-coder-7b-instruct-q4_k_m.gguf'
$keyFile = Join-Path $projectRoot 'runtime\access.key'
if (-not (Test-Path -LiteralPath $keyFile)) {
    [IO.Directory]::CreateDirectory((Split-Path -Parent $keyFile)) | Out-Null
    [IO.File]::WriteAllText($keyFile, ([Guid]::NewGuid().ToString('N') + [Guid]::NewGuid().ToString('N')))
}
$localHeaders = @{Authorization = 'Bearer ' + [IO.File]::ReadAllText($keyFile).Trim()}
function Test-Ready {
    try {
        $health = Invoke-RestMethod 'http://127.0.0.1:17861/health' -TimeoutSec 2
        $models = Invoke-RestMethod 'http://127.0.0.1:17861/v1/models' -Headers $localHeaders -TimeoutSec 2
        return ($health.status -eq 'ok' -and @($models.data.id) -contains 'boddy-1.0')
    } catch { return $false }
}
if (Test-Ready) { 'Boddy: motor local disponible.'; return }
if (-not (Test-Path -LiteralPath $server) -or -not (Test-Path -LiteralPath $model)) { throw 'Falta el motor o modelo. Ejecuta tools/Install-Model.ps1.' }
$lock = Get-Content -LiteralPath (Join-Path $projectRoot 'models.lock.json') -Raw | ConvertFrom-Json
if ((Get-Item -LiteralPath $model).Length -ne $lock.model_size) { throw 'Modelo incompleto: comprueba la descarga con Install-Model.ps1.' }
$arguments = '--model "' + $model + '" --alias boddy-1.0 --host 127.0.0.1 --port 17861 --ctx-size 8192 --parallel 1 --gpu-layers 99 --device Vulkan0 --flash-attn on --batch-size 256 --ubatch-size 128'
$arguments += ' --api-key-file "' + $keyFile + '" --cors-origins http://127.0.0.1:17861 --no-webui'
$process = Start-Process -FilePath $server -ArgumentList $arguments -WorkingDirectory (Split-Path -Parent $server) -WindowStyle Hidden -RedirectStandardOutput (Join-Path $projectRoot 'runtime\server.out.log') -RedirectStandardError (Join-Path $projectRoot 'runtime\server.err.log') -PassThru
[IO.File]::WriteAllText((Join-Path $projectRoot 'runtime\server.pid'), [string]$process.Id)
for ($i = 0; $i -lt 150; $i++) {
    if (Test-Ready) { 'Boddy: modelo listo en GPU.'; return }
    if ($process.HasExited) { throw ('El motor termino. Revisa runtime/server.err.log. Codigo: ' + $process.ExitCode) }
    Start-Sleep -Milliseconds 1000
}
if (-not $process.HasExited) { $process.Kill() }
throw 'El modelo no estuvo listo en el tiempo permitido. Revisa runtime/server.err.log.'

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$lock = Get-Content -LiteralPath (Join-Path $projectRoot 'models.lock.json') -Raw | ConvertFrom-Json
$runtimeDir = Join-Path $projectRoot 'runtime'
$modelDir = Join-Path $projectRoot 'models'
[IO.Directory]::CreateDirectory($runtimeDir) | Out-Null
[IO.Directory]::CreateDirectory($modelDir) | Out-Null
function Get-VerifiedFile($url, $target, $hash) {
    if (-not (Test-Path -LiteralPath $target)) {
        & curl.exe -L --fail --retry 2 --output $target $url
        if ($LASTEXITCODE -ne 0) { throw ('Descarga incompleta: ' + $target) }
    }
    if ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() -ne $hash) {
        throw ('Checksum incorrecto o descarga parcial: ' + $target + '. Conservado para revision; no se ejecutara.')
    }
}
[IO.Directory]::CreateDirectory((Join-Path $runtimeDir 'downloads')) | Out-Null
$zip = Join-Path $runtimeDir 'downloads\llama-b12110-vulkan.zip'
Get-VerifiedFile $lock.runtime_url $zip $lock.runtime_sha256
Get-VerifiedFile $lock.model_url (Join-Path $modelDir 'qwen2.5-coder-7b-instruct-q4_k_m.gguf') $lock.model_sha256
Expand-Archive -LiteralPath $zip -DestinationPath (Join-Path $runtimeDir 'llama') -Force
'Motor y modelo verificados por SHA256. Listos para iniciar.'

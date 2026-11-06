$ErrorActionPreference = 'Stop'
try {
    $projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
    Push-Location $projectRoot
    try {
        & cargo build -p boddy-cli --offline
        if ($LASTEXITCODE -ne 0) { throw 'No se pudo compilar Boddy.' }
        $engineMessage='Motor local disponible.'
        try { & (Join-Path $projectRoot 'tools\Start-Model.ps1') }
        catch { $engineMessage='Motor no disponible. Los controles locales siguen disponibles: '+$_.Exception.Message }
    } finally { Pop-Location }
    & (Join-Path $PSScriptRoot 'Boddy.ps1') -EngineStatus $engineMessage
} catch {
    Add-Type -AssemblyName System.Windows.Forms
    [void][Windows.Forms.MessageBox]::Show($_.Exception.Message, 'Boddy 1.0')
}

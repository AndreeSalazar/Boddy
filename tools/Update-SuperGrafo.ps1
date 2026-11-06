param([string]$SuperGrafoBin)

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
& node (Join-Path $PSScriptRoot 'update-architecture.cjs')
if ($LASTEXITCODE -ne 0) { throw 'Falló la validación de módulos.' }
if (-not $SuperGrafoBin) { $SuperGrafoBin = $env:BODDY_SG_BIN }
if (-not $SuperGrafoBin) {
    $SuperGrafoBin = Join-Path (Split-Path -Parent $projectRoot) 'SUPER GRAFO\ultra-carpetas\target\release\ultra-carpetas.exe'
}
if (-not (Test-Path -LiteralPath $SuperGrafoBin -PathType Leaf)) {
    throw 'No se encontró SUPER GRAFO; indica -SuperGrafoBin o BODDY_SG_BIN.'
}

# Script de mantenimiento para el dueño. El agente usa el adaptador Rust acotado.
$raw = & $SuperGrafoBin --consulta resumen $projectRoot
if ($LASTEXITCODE -ne 0) { throw 'Falló la consulta resumen de SUPER GRAFO.' }
$json = $raw -join "`n"
$summary = $json | ConvertFrom-Json
$destination = Join-Path $projectRoot 'super-grafo'
[IO.Directory]::CreateDirectory($destination) | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText((Join-Path $destination 'resumen.json'), $json + "`n", $utf8)

$map = @'
# Mapa de continuidad de Boddy

Generado por `tools/Update-SuperGrafo.ps1` a partir de una consulta real de SUPER
GRAFO y la descripción explícita de las capas de Boddy. No es una exportación
de la herramienta `guardar`: el ejecutable disponible aún no la implementa.

## Empezar aquí

1. Leer `docs/ESTADO.md`, `docs/PROPOSITO.md` y `docs/PLAN.md`.
2. Ejecutar `cargo run -p boddy-cli --offline -- grafo resumen` para datos actuales.
3. Consultar solo las partes necesarias y contrastar dependencias con Cargo.

## Capas

| Ruta | Papel | Depende de |
|---|---|---|
{{LAYERS}}
| `knowledge/sources.tsv` | Veinte fuentes: enlaces y metadatos | Fuentes oficiales consultadas |
| `docs` | Propósito, investigación, diseño, plan y estado | Decisiones y evidencia del proyecto |
| `tools/Update-SuperGrafo.ps1` | Actualizar este mapa y el resumen | Ejecutable externo de SUPER GRAFO |

## Última instantánea

Archivos indexados: {{FILES}}. Relaciones léxicas: {{DEPS}}. Ciclos reportados:
{{CYCLES}}. Fuente exacta de estas cifras: `super-grafo/resumen.json`.
Son una instantánea: generar este mapa agrega o modifica archivos y no produce
una fotografía inmutable de sí mismo. Consultar de nuevo para el estado actual.

El análisis es léxico y no equivale a una compilación o auditoría. Sus sospechas
no son defectos confirmados. Las comprobaciones de Rust están en `docs/ESTADO.md`.

## Lo que sigue

Estado vigente en docs/ESTADO.md y window/README.md. El chat local usa un motor
externo. Los laboratorios Tiny y Attention entrenan una MLP y un decoder causal
propios por WGPU con corpus versionado; Attention incorpora UTF-8 y embeddings;
no sustituye al chat. Ejecutor aislado y agente general siguen pendientes.
Las notas de cada nodo viven en `super-grafo/NOTAS.md`.
'@
$map = $map.Replace('{{FILES}}', [string]$summary.archivos_indexados)
$nodes = (Get-Content -LiteralPath (Join-Path $destination 'nodos.json') -Raw | ConvertFrom-Json).nodes
$layers = ($nodes | ForEach-Object { '| `' + $_.path + '` | ' + $_.purpose + ' (' + $_.status + ') | ' + ($_.dependencies -join ', ') + ' |' }) -join "`n"
$map = $map.Replace('{{LAYERS}}', $layers)
$map = $map.Replace('{{DEPS}}', [string]$summary.dependencias)
$map = $map.Replace('{{CYCLES}}', [string]$summary.ciclos.total)
[IO.File]::WriteAllText((Join-Path $destination 'MAPA.md'), $map + "`n", $utf8)
Write-Output 'Actualizados super-grafo/MAPA.md y super-grafo/resumen.json.'

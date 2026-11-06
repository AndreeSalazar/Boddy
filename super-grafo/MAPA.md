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
| `crates/boddy-app` | Casos de uso de la aplicacion (implemented) | boddy-catalog, boddy-core, boddy-inference, boddy-supergrafo, boddy-telemetry |
| `crates/boddy-catalog` | Catalogo documental local (implemented) |  |
| `crates/boddy-core` | Estados, cupos y evidencia (implemented) | boddy-contracts, boddy-mission, boddy-policy |
| `crates/boddy-contracts` | Errores compartidos sin plataforma (implemented) |  |
| `crates/boddy-mission` | Razon, objetivo y aceptacion (implemented) | boddy-contracts |
| `crates/boddy-policy` | Capacidades explicitas (implemented) |  |
| `crates/boddy-inference` | Chat con motor local por loopback, sin herramientas (implemented) |  |
| `crates/boddy-supergrafo` | Consultas externas acotadas (implemented) |  |
| `crates/boddy-telemetry` | Muestra de VRAM y uso de GPU, independiente del modelo (implemented) |  |
| `crates/boddy-cli` | Entrada de terminal (implemented) | boddy-app |
| `window` | Lanzador grafico Windows sin consola (implemented) | boddy-cli |
| `modules/inference` | Frontera de inference (planned) |  |
| `modules/tokenizer` | Frontera de tokenizer (planned) |  |
| `modules/ingestion` | Frontera de ingestion (planned) |  |
| `modules/retrieval` | Frontera de retrieval (planned) |  |
| `modules/session` | Frontera de session (planned) |  |
| `modules/executor` | Frontera de executor (planned) |  |
| `modules/verification` | Frontera de verification (planned) |  |
| `modules/evaluation` | Frontera de evaluation (planned) |  |
| `gpu/wgpu` | Adaptador GPU sin filtrar tipos al nucleo (planned) |  |
| `knowledge/sources.tsv` | Veinte fuentes: enlaces y metadatos | Fuentes oficiales consultadas |
| `docs` | Propósito, investigación, diseño, plan y estado | Decisiones y evidencia del proyecto |
| `tools/Update-SuperGrafo.ps1` | Actualizar este mapa y el resumen | Ejecutable externo de SUPER GRAFO |

## Última instantánea

Archivos indexados: 103. Relaciones léxicas: 9. Ciclos reportados:
0. Fuente exacta de estas cifras: `super-grafo/resumen.json`.
Son una instantánea: generar este mapa agrega o modifica archivos y no produce
una fotografía inmutable de sí mismo. Consultar de nuevo para el estado actual.

El análisis es léxico y no equivale a una compilación o auditoría. Sus sospechas
no son defectos confirmados. Las comprobaciones de Rust están en `docs/ESTADO.md`.

## Lo que sigue

Estado vigente en docs/ESTADO.md y window/README.md. El chat local usa un motor
externo; WGPU propio, corpus versionado, ejecutor aislado y ciclo del agente siguen pendientes.
Las notas de cada nodo viven en `super-grafo/NOTAS.md`.

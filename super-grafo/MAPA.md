# Mapa de continuidad de Boddy

Generado por `tools/Update-SuperGrafo.ps1` a partir de una consulta real de SUPER
GRAFO y la descripciÃ³n explÃ­cita de las capas de Boddy. No es una exportaciÃ³n
de la herramienta `guardar`: el ejecutable disponible aÃºn no la implementa.

## Empezar aquÃ­

1. Leer `docs/ESTADO.md`, `docs/PROPOSITO.md` y `docs/PLAN.md`.
2. Ejecutar `cargo run -p boddy-cli --offline -- grafo resumen` para datos actuales.
3. Consultar solo las partes necesarias y contrastar dependencias con Cargo.

## Capas

| Ruta | Papel | Depende de |
|---|---|---|
| `crates/boddy-agent` | Investigador con busqueda de componentes y una correccion de citas verificada (implemented) | boddy-core, boddy-inference, boddy-reader, boddy-supergrafo |
| `crates/boddy-core` | Estados, cupos y evidencia (implemented) | boddy-contracts, boddy-mission, boddy-policy |
| `crates/boddy-contracts` | Errores compartidos sin plataforma (implemented) |  |
| `crates/boddy-mission` | Razon, objetivo y aceptacion (implemented) | boddy-contracts |
| `crates/boddy-policy` | Capacidades explicitas (implemented) |  |
| `crates/boddy-inference` | Chat con motor local por loopback, sin herramientas (implemented) |  |
| `crates/boddy-reader` | Lectura de fuentes acotada y evidencia con SHA256 (implemented) |  |
| `crates/boddy-supergrafo` | Consultas externas acotadas (implemented) |  |
| `crates/boddy-app` | Casos de uso de la aplicacion (implemented) | boddy-agent, boddy-catalog, boddy-core, boddy-desktop, boddy-inference, boddy-lab, boddy-reader, boddy-supergrafo, boddy-telemetry |
| `crates/boddy-catalog` | Catalogo documental local (implemented) |  |
| `crates/boddy-desktop` | Acciones manuales de escritorio registradas y auditadas (implemented) | boddy-reader |
| `crates/boddy-lab` | Entrenamiento reanudable, registro de modelos, control no entrenado y generacion Tiny CPU (implemented) | boddy-neural, boddy-reader, boddy-wgpu |
| `crates/boddy-neural` | MLP no_std de siguiente caracter y gradientes CPU de referencia (implemented) |  |
| `gpu/wgpu` | Entrenamiento MLP con shaders propios WGPU y tipos GPU privados (implemented) | boddy-neural |
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
| `knowledge/sources.tsv` | Veinte fuentes: enlaces y metadatos | Fuentes oficiales consultadas |
| `docs` | PropÃ³sito, investigaciÃ³n, diseÃ±o, plan y estado | Decisiones y evidencia del proyecto |
| `tools/Update-SuperGrafo.ps1` | Actualizar este mapa y el resumen | Ejecutable externo de SUPER GRAFO |

## Ãšltima instantÃ¡nea

Archivos indexados: 152. Relaciones lÃ©xicas: 17. Ciclos reportados:
0. Fuente exacta de estas cifras: `super-grafo/resumen.json`.
Son una instantÃ¡nea: generar este mapa agrega o modifica archivos y no produce
una fotografÃ­a inmutable de sÃ­ mismo. Consultar de nuevo para el estado actual.

El anÃ¡lisis es lÃ©xico y no equivale a una compilaciÃ³n o auditorÃ­a. Sus sospechas
no son defectos confirmados. Las comprobaciones de Rust estÃ¡n en `docs/ESTADO.md`.

## Lo que sigue

Estado vigente en docs/ESTADO.md y window/README.md. El chat local usa un motor
externo. El laboratorio Tiny entrena una MLP propia por WGPU con corpus versionado;
no sustituye al chat. Ejecutor aislado y agente general siguen pendientes.
Las notas de cada nodo viven en `super-grafo/NOTAS.md`.

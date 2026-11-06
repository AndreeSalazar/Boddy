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
| `crates/boddy-agent` | Investigador de lectura y controlador Rust de ranking, propuesta y verificacion acotada (implemented) | boddy-core, boddy-inference, boddy-reader, boddy-rust-tasks, boddy-supergrafo |
| `crates/boddy-core` | Estados, cupos y evidencia (implemented) | boddy-contracts, boddy-mission, boddy-policy |
| `crates/boddy-contracts` | Errores compartidos sin plataforma (implemented) |  |
| `crates/boddy-mission` | Razon, objetivo y aceptacion (implemented) | boddy-contracts |
| `crates/boddy-policy` | Capacidades explicitas (implemented) |  |
| `crates/boddy-inference` | Chat con motor local por loopback, sin herramientas (implemented) |  |
| `crates/boddy-reader` | Lectura de fuentes acotada y evidencia con SHA256 (implemented) |  |
| `crates/boddy-rust-tasks` | Curriculo Rust y ejercicios tipados no_std con verificador finito de expresiones cerradas (implemented) |  |
| `crates/boddy-supergrafo` | Consultas externas acotadas (implemented) |  |
| `crates/boddy-app` | Casos de uso de la aplicacion (implemented) | boddy-agent, boddy-catalog, boddy-core, boddy-desktop, boddy-inference, boddy-lab, boddy-library, boddy-reader, boddy-supergrafo, boddy-telemetry |
| `crates/boddy-catalog` | Catalogo documental local (implemented) |  |
| `crates/boddy-desktop` | Acciones manuales de escritorio registradas y auditadas (implemented) | boddy-reader |
| `crates/boddy-lab` | Experimentos Tiny y Attention: corpus, entrenamiento, evaluacion, registro y checkpoints propios (implemented) | boddy-neural, boddy-reader, boddy-rust-tasks, boddy-transformer, boddy-wgpu |
| `crates/boddy-neural` | MLP no_std de siguiente caracter y gradientes CPU de referencia (implemented) |  |
| `crates/boddy-transformer` | Decoder no_std con embeddings, atencion causal, RMSNorm y autodiferenciacion de referencia (implemented) |  |
| `gpu/wgpu` | Entrenamiento MLP y cinta Transformer con shaders propios y tipos WGPU privados (implemented) | boddy-neural, boddy-transformer |
| `crates/boddy-library` | Biblioteca local: registro, recuperacion lexica por presupuesto, recuerdos y exportacion con procedencia (implemented) | boddy-reader, library-data |
| `crates/boddy-telemetry` | Muestra de VRAM y uso de GPU, independiente del modelo (implemented) |  |
| `crates/boddy-cli` | Entrada de terminal (implemented) | boddy-app |
| `window` | Lanzador grafico Windows sin consola (implemented) | boddy-cli |
| `biblioteca/01-forja` | La Forja: Programacion: escuelas, fundamentos, motivos y comprobaciones (partial) |  |
| `biblioteca/02-babel` | Babel: Idiomas naturales: catalogo IANA y escuelas documentales ampliables (partial) |  |
| `biblioteca/03-agora` | Agora: Razonamiento: hipotesis, evidencia y contradicciones (partial) |  |
| `biblioteca/04-atelier` | Atelier: Creacion: problemas, alternativas y criterios de decision (partial) |  |
| `biblioteca/05-bitacora` | Bitacora: Memoria documental: referencias, contexto y relectura (partial) |  |
| `biblioteca` | Registro de fichas, relaciones documentales y catalogo IANA; contenido parcial, no pesos entrenados (partial) | biblioteca-forja, biblioteca-babel, biblioteca-agora, biblioteca-atelier, biblioteca-bitacora |
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

Archivos indexados: 216. Relaciones lÃ©xicas: 26. Ciclos reportados:
0. Fuente exacta de estas cifras: `super-grafo/resumen.json`.
Son una instantÃ¡nea: generar este mapa agrega o modifica archivos y no produce
una fotografÃ­a inmutable de sÃ­ mismo. Consultar de nuevo para el estado actual.

El anÃ¡lisis es lÃ©xico y no equivale a una compilaciÃ³n o auditorÃ­a. Sus sospechas
no son defectos confirmados. Las comprobaciones de Rust estÃ¡n en `docs/ESTADO.md`.

## Lo que sigue

Estado vigente en docs/ESTADO.md y window/README.md. El chat local usa un motor
externo. Los laboratorios Tiny y Attention entrenan una MLP y un decoder causal
propios por WGPU con corpus versionado; Attention incorpora UTF-8 y embeddings;
no sustituye al chat. Ejecutor aislado y agente general siguen pendientes.
La escuela Rust aÃ±ade entrenamiento de patrones y un ciclo acotado de propuesta
y verificacion entre expresiones conocidas, sin ejecutar proyectos.
Las notas de cada nodo viven en `super-grafo/NOTAS.md`.

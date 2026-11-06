# Estado para continuar

Fecha de inicio: 5 de noviembre de 2026, America/Lima.

## Decisiones del dueño

- Construir una IA operativa personal desde sus fundamentos en Rust y `no_std`.
- Usar SUPER GRAFO para orientarse y mantener el contexto del proyecto.
- Investigar publicaciones públicas de Anthropic como inspiración.
- Convertir conocimiento y herramientas de Python en capacidades útiles.
- Hardware comunicado: RTX 3060 de 12 GB, 16 GB de RAM.

## Modularización posterior al commit inicial

Base inicial guardada en `091dc56`. Ahora hay ocho crates independientes:
contracts, mission, policy, core, catalog, supergrafo, app y cli.
Contratos, misión, política y núcleo siguen siendo no_std.
El cuadro Windows está en `window/Boddy.ps1`; prueba de controles y ejecución
de búsqueda Python aprobada con Windows PowerShell en STA.
Registro de 18 nodos con dependencias acíclicas y posiciones por capa;
WGPU y los otros componentes futuros figuran como pendientes.
Esto separa código y responsabilidades: no establece aislamiento de procesos del SO.

## Implementado

- Workspace Rust modular, sin paquetes externos; `boddy-cli` consume `boddy-app`.
- Misión con razón, objetivo y aceptación; estados y referencias de evidencia acotadas.
- Política inicial que deniega ejecución de código y red; cupo por tarea en memoria.
- Verificación explícita requerida para completar; revisión invalida resultados anteriores.
- Catálogo de veinte fuentes primarias con búsqueda de metadatos.
- Adaptador a SUPER GRAFO con tres consultas de lectura, límite de salida y timeout.
- Documentos de propósito, arquitectura, investigación, Python y plan.

## Comprobaciones

- `cargo test --workspace --offline`: 13 pruebas aprobadas (9 contratos del núcleo y 4 del host).
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: aprobado.
- `cargo check -p boddy-core --target wasm32-unknown-unknown --offline`: aprobado.
- `cargo run -p boddy-cli --offline -- fuentes python`: devuelve las siete referencias Python del catálogo.
- `cargo run -p boddy-cli --offline -- demo`: deniega código y completar sin verificación.
- `cargo run -p boddy-cli --offline -- grafo resumen`: consulta real al ejecutable existente, salida JSON correcta.
- `tools/Update-SuperGrafo.ps1`: ejecutado; creó el mapa de continuidad y la instantánea JSON.

El check Wasm comprueba portabilidad de compilación de la librería; no demuestra
un arranque bare metal ni ejecuta código en GPU. La terminación por timeout está
implementada, pero no se ha sometido todavía a una prueba de integración que fuerce
30 segundos de bloqueo. Se probaron el límite de bytes y la lista de consultas.

El ejecutable disponible de SUPER GRAFO responde a `resumen` pero rechaza `guardar`
como herramienta inexistente. Se usa su resumen real y un mapa de continuidad
generado por `tools/Update-SuperGrafo.ps1`, sin modificar ni recompilar SUPER GRAFO.
No se interpretan sus sospechas léxicas de elegancia como errores confirmados.

## Pendiente

No hay pesos, tokenizer, inferencia, shaders de cómputo, corpus descargado, embeddings,
intérprete Python, generación de parches, sandbox del SO ni registro persistente de
sesiones del agente. No hay benchmark en GPU. Los tests del controlador no miden
inteligencia o calidad de programación. El host puede aportar referencias falsas:
su verificador es parte de la base de confianza.

## Retomar

Leer primero `super-grafo/MAPA.md`, este archivo y `docs/PLAN.md`. Consultar
`cargo run -p boddy-cli --offline -- grafo resumen` antes de bajar a archivos.
Mantener lo observado separado de lo propuesto; registrar evidencia de cada capacidad.

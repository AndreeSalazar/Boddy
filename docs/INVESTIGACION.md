# Investigación aplicada — 5 de noviembre de 2026

Primera revisión de fuentes oficiales. Es una selección inicial, no un inventario
de todas las IA ni una reconstrucción de sus implementaciones privadas. Las
decisiones de Boddy que siguen son nuestra adaptación al uso personal y al hardware.
El catálogo está en `knowledge/sources.tsv`: enlaces y metadatos, sin corpus ingerido.

## Anthropic: de publicación a decisión

| Fuente primaria | Observación de la publicación | Aplicación propia en Boddy |
|---|---|---|
| [Building effective agents](https://www.anthropic.com/research/building-effective-agents), 19/12/2024 | Diferencia flujos prefijados y agentes que deciden con herramientas. | Un controlador explícito para permisos y estados; el modelo propondrá acciones dentro de ese contrato. |
| [Writing effective tools](https://www.anthropic.com/research/writing-tools-for-agents), 11/09/2025 | Trata diseño, prototipos y evaluación de herramientas. | Empezar con consultas concretas de SUPER GRAFO y salidas acotadas. |
| [Context engineering](https://www.anthropic.com/research/effective-context-engineering-for-ai-agents), 29/09/2025 | El contexto se selecciona y mantiene durante la inferencia. | Recuperar archivos y documentos según la tarea; conservar referencias para volver al original. |
| [Long-running agents](https://www.anthropic.com/research/effective-harnesses-for-long-running-agents), 26/11/2025 | Describe progreso incremental y artefactos para continuar entre sesiones. | Mantener estado, criterios y pendientes en el repositorio; incorporar persistencia de sesiones más adelante. |
| [Code execution with MCP](https://www.anthropic.com/research/code-execution-with-mcp), 04/11/2025 | Propone procesar datos cerca de las herramientas para reducir tráfico al modelo. | Filtrar resultados en el host y enviar al modelo solo lo necesario; ejecución de código pendiente del sandbox. |
| [Demystifying evals](https://www.anthropic.com/research/demystifying-evals-for-ai-agents), 09/01/2026 | Evalúa el resultado y la trayectoria de agentes que usan herramientas. | Distinguir pruebas del controlador, pruebas del modelo y tareas completas del agente. |
| [Managed Agents](https://www.anthropic.com/research/managed-agents), 08/04/2026 | Separa sesión, bucle de agente y entorno de ejecución mediante interfaces. | Diseñar contratos independientes para memoria, inferencia y ejecutor; el modelo será sustituible. |
| [Harness design](https://www.anthropic.com/research/harness-design-long-running-apps), 24/03/2026 | Describe planificación, generación y evaluación en trabajo prolongado. | Aplicar primero esas fases de forma secuencial, con comprobaciones independientes del texto generado. |

El artículo de 2024 remite ahora a Managed Agents como evolución de su enfoque.
Por eso no tratamos una receta antigua como arquitectura definitiva. La organización
de fases en Boddy tampoco implica que hoy haya varios modelos o agentes ejecutándose.

## Rust, GPU y aislamiento

[Rust Embedded Book](https://doc.rust-lang.org/embedded-book/intro/no-std.html)
explica el entorno sin biblioteca estándar. Aplicación: separar lógica de dominio
y servicios del sistema. Un núcleo `no_std` no vuelve bare metal al proceso de Windows.

[wgpu](https://docs.rs/wgpu/latest/wgpu/) expone backends gráficos y una feature
`std` que puede desactivarse. **La viabilidad de un backend concreto sin `std`
debe comprobarse**; no equivale a disponer de controladores GPU propios sin SO.
Aquí el núcleo ya es `no_std`; la futura integración de inferencia está por validar.

[Burn](https://burn.dev/books/burn/basic-workflow/backend.html) permite elegir
un backend WGPU. Lo evaluaremos con el modelo, formatos y operaciones concretos;
que exista ese backend no demuestra compatibilidad con cualquier LLM o cuantización.

[Microsoft](https://learn.microsoft.com/en-us/windows/security/book/application-security-application-isolation)
describe AppContainer como frontera de aislamiento y Windows Sandbox como entorno
virtualizado. La decisión exacta necesita una prueba de acceso a disco, red y GPU;
un subproceso normal no cumple ese requisito.

## Modelo y recursos: hipótesis a comprobar

La [ficha oficial de Qwen2.5-Coder-7B-Instruct](https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct)
sirve como candidato de referencia para programación, con licencia Apache-2.0.
No está seleccionado como ganador ni descargado. Habrá que comparar alternativas
y verificar compatibilidad de backend, formato de pesos, tokenizer y plantilla de chat.

Estimación propia: 7–8 mil millones de parámetros a 4 bits ocupan aproximadamente
3,5–4 GB solo en valores de pesos. Escalas, buffers, caché KV y contexto agregan
memoria. Esa cuenta no prueba que un modelo completo quepa ni predice su velocidad.

Con 12 GB de VRAM y 16 GB de RAM, arrancaremos las mediciones con un solo modelo y
contexto moderado. Objetivos iniciales orientativos: menos de 8 GiB de VRAM y 6 GiB
de RAM del proceso. No son límites implementados ni resultados medidos. Se registrarán
consumo pico, tiempo al primer token, tokens/s, formato y longitud de contexto.

# Plan de construcción

Actualización Boddy 1.0: existe un adaptador de chat a un motor externo local
(llama.cpp/Vulkan) y una interfaz Windows. No sustituye la entrega de inferencia
propia WGPU ni el ejecutor aislado. Ver `window/README.md` y `docs/ESTADO.md`.

Prioridad elegida por el dueño: aprender entrenamiento desde cero. Implementado
un laboratorio MLP con pesos propios, corpus pequeño, WGPU y referencia CPU.
No completa la entrega del asistente general. Ver MODELO-PROPIO.md.

6 de noviembre: disponible el primer investigador de lectura con núcleo, selección
de fuentes, citas verificadas y registro de tarea. Es un subconjunto de la entrega
4; planes abiertos, reanudación y ejecución siguen pendientes. Ver AGENTE-LECTURA.md.

Cada entrega exige comportamiento observable. La casilla de una capacidad solo
se completa con evidencia pertinente; una demo determinista no cuenta como IA.

| Entrega | Resultado | Aceptación | Estado |
|---|---|---|---|
| 0. Fundamentos | Propósito, investigación, mapa y controlador `no_std` | Compilación, contratos de permisos/estados y consulta real a SUPER GRAFO | Implementada; ver comprobaciones en ESTADO.md |
| 1. Conocimiento | Un pequeño corpus Python y Rust versionado | Consultas encuentran fragmentos correctos y muestran versión/origen; medición de recuperación | Pendiente |
| 2. Inferencia | Un modelo local, tokenizer y backend compatibles | Generación real; memoria y velocidad medidas en RTX 3060; comparación con referencia | Pendiente |
| 3. Ejecutor | Herramientas restringidas en Windows | Pruebas positivas y negativas de disco, red, procesos, cancelación y recursos | Pendiente |
| 4. Agente | Ciclo modelo-herramientas-verificador con sesión persistente | Retoma una tarea, respeta permisos/cupos y no aprueba cambios fallidos | Pendiente |
| 5. Programación útil | Localiza y corrige un fallo en un proyecto de prueba | Parche aplicable y prueba de regresión, con criterio externo al modelo | Pendiente |

Las entregas 1 y 2 pueden desarrollarse independientemente; la ejecución de código
del agente necesita antes la entrega 3. Esta descripción no implica delegación o
ejecución simultánea de varios agentes.

## Siguiente paso concreto

Para el modelo propio: Attention 0.1 ya prueba UTF-8, embeddings y atención causal
en un decoder educativo (ATTENTION.md). Lo siguiente es corpus y tareas de
evaluación independientes; medir CPU/cinta GPU antes de optimizar kernels.
La escuela Rust añade un corpus inicial y búsqueda verificada de expresiones
conocidas (ESCUELA-RUST.md). Programación de proyectos, sesiones reanudables y
ejecución aislada siguen pendientes; los seis ejercicios no completan esas entregas.
Documentación Python recuperable sigue siendo un objetivo distinto: corpus fijado
a una versión, referencias y diez preguntas contrastables antes de medir recuperación.

La biblioteca ya ofrece recuperacion lexica por registro, notas introductorias
Rust, catalogo IANA, referencias persistentes y exportacion acotada. Ver BIBLIOTECA.md.
Cursos completos, recuperacion semantica e inyeccion automatica a modelos siguen
pendientes. La biblioteca actual no completa por si sola la entrega de conocimiento.

## Cómo medir la IA cuando exista

- Investigación: exactitud de respuesta, soporte de las citas y recuperación de la sección correcta.
- Programación: tareas resueltas con pruebas independientes, regresiones y cambios innecesarios.
- Operación: acciones fuera de alcance, cancelación, recuperación tras reinicio y respeto de cupos.
- Recursos: RAM/VRAM pico, tiempo al primer token y latencia completa por tarea.

Comparar con la misma versión de tareas, contexto, herramientas y hardware. Repetir
las evaluaciones del modelo cuando haya aleatoriedad; una sola respuesta acertada
no establece precisión general. No se declara paridad con modelos comerciales.

# Plan de construcción

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

Construir un corpus pequeño de documentación Python fijado a una versión, con
referencias y consultas léxicas. Elegir diez preguntas con respuestas contrastables
antes de medir recuperación. Después comparar candidatos de modelo y compatibilidad
WGPU con un contexto pequeño; no descargar varios modelos a ciegas.

## Cómo medir la IA cuando exista

- Investigación: exactitud de respuesta, soporte de las citas y recuperación de la sección correcta.
- Programación: tareas resueltas con pruebas independientes, regresiones y cambios innecesarios.
- Operación: acciones fuera de alcance, cancelación, recuperación tras reinicio y respeto de cupos.
- Recursos: RAM/VRAM pico, tiempo al primer token y latencia completa por tarea.

Comparar con la misma versión de tareas, contexto, herramientas y hardware. Repetir
las evaluaciones del modelo cuando haya aleatoriedad; una sola respuesta acertada
no establece precisión general. No se declara paridad con modelos comerciales.

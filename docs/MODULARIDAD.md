# Fronteras y reglas de modularidad

Las reglas persistentes están en `AGENTS.md`. Modularidad significa contratos y
dependencias comprobables; el aislamiento de procesos del SO es otro requisito.
Las crates actuales comparten proceso y no constituyen sandboxes independientes.

| Componente | Responsabilidad única | Dependencias internas |
|---|---|---|
| boddy-contracts | Errores compartidos, no_std | Ninguna |
| boddy-mission | Porqué, objetivo y aceptación, no_std | contracts |
| boddy-policy | Capacidades, no_std | Ninguna |
| boddy-core | Estados, evidencia y cupos, no_std | contracts, mission, policy |
| boddy-catalog | Metadatos documentales | Ninguna |
| boddy-supergrafo | Proceso externo y consultas acotadas | Ninguna |
| boddy-app | Casos de uso y comandos | core, catalog, supergrafo, inference, telemetry, reader, agent, desktop, lab |
| boddy-cli | Entrada/salida de terminal | app |
| boddy-inference | Cliente del modelo local, sin herramientas | serde_json; motor externo por loopback |
| boddy-telemetry | Muestra de VRAM y uso del dispositivo | serde_json; nvidia-smi externo |
| boddy-reader | Fuentes acotadas, rutas y SHA256 | serde_json, sha2; disco del host |
| boddy-desktop | Aperturas manuales registradas del escritorio | reader; procesos locales sin shell |
| boddy-agent | Investigación de lectura y registro de tareas | core, inference, reader, supergrafo |
| window | Cuadro de comandos Windows | CLI por proceso |
| boddy-neural | MLP, gradientes y referencia CPU no_std | libm; alloc del consumidor |
| boddy-transformer | Decoder causal, cinta y autodiferenciación no_std | libm; alloc del consumidor |
| gpu/wgpu (boddy-wgpu) | MLP y cinta Transformer en GPU | neural, transformer; WGPU privado |
| boddy-lab | Experimentos, corpus y checkpoints | neural, transformer, wgpu, reader |

Existe un adaptador a llama.cpp, una MLP propia y un decoder causal educativo con
UTF-8/embeddings/WGPU, separados del chat. Módulos futuros del asistente general:
inferencia propia completa, tokenizer de subpalabras, ingestión, recuperación,
sesiones persistentes, ejecución aislada, verificación y evaluaciones del agente.
Se distinguen en el registro de nodos como pendientes, sin crates vacías que
prometan funcionalidad. Su implementación deberá fijar entradas, salidas, efectos,
dependencias, límites y pruebas antes de conectarse a la aplicación.

Optimización inicial comprobable: el núcleo y las herramientas se compilan sin
UI ni bibliotecas GPU, el catálogo no necesita procesos y la ventana solo consume
la aplicación. No se afirma una mejora de velocidad sin un benchmark comparable.

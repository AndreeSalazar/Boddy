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
| boddy-app | Casos de uso y comandos | core, catalog, supergrafo |
| boddy-cli | Entrada/salida de terminal | app |
| boddy-inference | Cliente del modelo local, sin herramientas | serde_json; motor externo por loopback |
| boddy-telemetry | Muestra de VRAM y uso del dispositivo | serde_json; nvidia-smi externo |
| boddy-reader | Fuentes acotadas, rutas y SHA256 | serde_json, sha2; disco del host |
| boddy-desktop | Aperturas manuales registradas del escritorio | reader; procesos locales sin shell |
| boddy-agent | Investigación de lectura y registro de tareas | core, inference, reader, supergrafo |
| window | Cuadro de comandos Windows | CLI por proceso |
| gpu/wgpu | Backend GPU pendiente | Contrato futuro con host |

Existe un primer adaptador de inferencia a llama.cpp; no una implementación propia
de redes neuronales o WGPU. Módulos futuros separados: inferencia propia, tokenizer, ingestión, recuperación,
sesiones persistentes, ejecución aislada, verificación y evaluaciones del agente.
Se distinguen en el registro de nodos como pendientes, sin crates vacías que
prometan funcionalidad. Su implementación deberá fijar entradas, salidas, efectos,
dependencias, límites y pruebas antes de conectarse a la aplicación.

Optimización inicial comprobable: el núcleo y las herramientas se compilan sin
UI ni bibliotecas GPU, el catálogo no necesita procesos y la ventana solo consume
la aplicación. No se afirma una mejora de velocidad sin un benchmark comparable.

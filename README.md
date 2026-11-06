# Boddy 1.0

Asistente personal de Eddi Salazar. Chat local sencillo, Rust modular y núcleo
no_std. Modelo base: Qwen2.5-Coder-7B-Instruct Q4_K_M; motor externo llama.cpp
b12110, con Vulkan en la RTX 3060. No son pesos entrenados por este proyecto.

**Laboratorio** añade Boddy Tiny 1.0: una red de 27.776 parámetros entrenada desde
cero en la RTX 3060 por WGPU, con pesos propios, corpus versionado, contadores y
checkpoints. Es un experimento educativo de caracteres. [Cómo entrenarlo](docs/MODELO-PROPIO.md).

## Abrir el chat

Doble clic en `window/Boddy.vbs`, o:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -STA -File .\window\Start-Boddy.ps1
```

Enter envía, Shift+Enter agrega una línea. Nuevo chat borra el historial en memoria.
Activa **Investigar proyecto (lectura)** para un objetivo como «Explica cómo
budget.rs limita los tokens». Boddy buscará fuentes, leerá fragmentos y entregará
un informe con citas comprobadas y ruta del registro local.
El chat no tiene herramientas para leer tus archivos o ejecutar programas.
La respuesta se muestra al finalizar; el historial reciente se recorta con límites.

Primera preparación en otro equipo: `tools/Install-Model.ps1` descarga motor y
pesos oficiales y verifica SHA256. Son unos 4,7 GB de modelo, conservados fuera de
Git. `models.lock.json` fija formato, origen y hashes. El motor permanece cargado;
`tools/Stop-Model.ps1` lo detiene para liberar recursos. [Detalles](window/README.md).

## Arquitectura

Diecisiete crates: contratos, misión, permisos, controlador, catálogo, SUPER GRAFO,
inferencia local, telemetría, lector, agente, escritorio, aplicación, CLI, lanzador
gráfico, red neuronal de referencia, adaptador WGPU y laboratorio.
Las primeras cuatro permanecen no_std.
El host usa serde_json; GPU y ventana no entran en el núcleo.

El chat usa Vulkan mediante un proceso externo; el laboratorio usa WGPU propio.
El sandbox del SO y la ejecución operativa de tareas siguen pendientes.
Los tokens son unidades internas de texto del modelo; el cómputo es local,
sin clave ni pago a un proveedor por token.

- [Reglas](AGENTS.md) y [fronteras](docs/MODULARIDAD.md).
- [Propósito](docs/PROPOSITO.md), [investigación](docs/INVESTIGACION.md) y [plan](docs/PLAN.md).
- [Estado y comprobaciones](docs/ESTADO.md).
- [Tokens, GPU y presupuestos](docs/RECURSOS.md).
- [Agente de lectura con citas verificadas](docs/AGENTE-LECTURA.md).
- [Programar, Mi PC y Evidencia en Windows](docs/ESCRITORIO.md).
- [Evolución y evaluaciones del agente](docs/EVOLUCION.md).
- [Mapa](super-grafo/MAPA.md) y `super-grafo/nodos.json`.

## Comprobar y mantener

```powershell
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo check -p boddy-core --target wasm32-unknown-unknown --offline
cargo run -p boddy-cli --offline -- fuentes python
cargo run -p boddy-cli --offline -- grafo resumen
.\tools\Update-SuperGrafo.ps1
```

El adaptador del chat solo conecta a 127.0.0.1:17861. No hace peticiones de
inferencia a servidores remotos. El catálogo sigue conteniendo metadatos y enlaces,
no un corpus descargado. Actualizar el mapa requiere Node.js y Cargo.

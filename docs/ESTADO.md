# Estado vigente: Boddy 1.0

Hardware observado: RTX 3060, 12288 MiB, controlador NVIDIA 610.74; RAM comunicada:
16 GB. Los commits 091dc56 y c9aa594 conservan las etapas iniciales del proyecto.

## Disponible

- Chat Windows sencillo: historial reciente, Enter para enviar, Shift+Enter para
  nueva línea, nuevo chat y cancelación de la espera.
- Catorce crates con dependencias acíclicas. Contratos, misión, política y controlador
  siguen siendo no_std; host, inferencia y ventana permanecen separados.
- Adaptador Rust boddy-inference a un motor local externo, solo 127.0.0.1:17861.
- Qwen2.5-Coder-7B-Instruct Q4_K_M, 4683073536 bytes. SHA256 verificado contra
  metadatos oficiales de Hugging Face; motor llama.cpp b12110 Vulkan verificado
  contra digest oficial de GitHub. Origen y hashes en models.lock.json.
- El motor detectó Vulkan0: NVIDIA GeForce RTX 3060. Modelo cargado con capacidad
  de 8192 tokens. Presupuesto por petición configurable, por defecto 2048 tokens
  incluyendo reserva de respuesta; salida por defecto 256, hasta 1024.
- Autenticación local interna generada automáticamente; no es una clave de proveedor.
  Los pesos, motor, credencial y logs están en carpetas ignoradas por Git.
- Catálogo de veinte referencias documentales; todavía son metadatos, sin corpus.
- SUPER GRAFO: mapa, notas, posiciones y 23 nodos con estados explícitos.
- Contadores visibles de entrada, salida, caché y acumulados, velocidad, recorte
  e instantánea de VRAM del dispositivo al responder. Conteo previo por tokenizer
  del motor, con eliminación de pares antiguos para respetar el presupuesto.
- Casilla opcional Mapa de Boddy: el host consulta un resumen del proyecto y solo
  entrega cifras seleccionadas; no permite al modelo ejecutar herramientas.
- Modo Investigar proyecto: conecta misión, estados y cupo del núcleo a búsquedas
  y lecturas decididas por el modelo dentro de un flujo acotado. Hasta tres fuentes;
  informe con líneas/citas comprobadas y SHA256 estable. Registro en data/tasks,
  excluido de Git. Revisión semántica pendiente incluso cuando termina Completed.
- Ventana con Conversar, Programar, Mi PC y Evidencia. Explorador acotado de
  fuentes (incluye window como texto), fragmentos y apertura manual en editor.
  Acciones fijas para Explorer, SUPER GRAFO y Administrador de tareas, con log local.
  Vista de tareas guardadas y consulta manual de GPU, independiente del modelo.

## Verificado en esta entrega

- cargo test --workspace --offline: 24 pruebas aprobadas.
- cargo clippy --workspace --all-targets --offline -- -D warnings: aprobado.
- cargo check -p boddy-core --target wasm32-unknown-unknown --offline: aprobado.
- Comprobación del evento del botón de la ventana con generación real: respondió
  «¡Hola!» a la petición de saludo en español.
- Dos turnos por el cliente Rust: respondió «Entendido.» y recuperó correctamente
  «FaroAzul» del historial de prueba. No se atribuye al modelo memoria persistente.
- Investigación real sobre budget.rs: Completed, tres llamadas al modelo, citas
  textuales comprobadas por el host. Pasó también desde la ventana y no alteró
  el historial del chat. Una prueba sin motor terminó Stopped con registro local.
- Las cuatro pestañas cargaron fuentes, fragmento, un registro y estado de GPU.
  La investigación desde Programar también mostró una cita incorrecta del modelo:
  quedó Stopped y visible como no aprobada. Las aperturas externas se verificaron
  como planes de argumentos, sin automatizar esas aplicaciones durante las pruebas.
- VRAM total observada tras cargar y generar: aproximadamente 5,5–5,6 GiB,
  incluyendo el uso previo del escritorio. No es consumo aislado del proceso,
  una medición del pico ni un benchmark de calidad o velocidad general.
- Lanzador nativo compilado sin consola; proceso Windows con título Boddy 1.0.
  La herramienta visual no expuso sus ventanas PowerShell; la verificación de
  interfaz se limita al test nativo de controles/eventos. DrawToBitmap no captura
  correctamente RichTextBox y su vista previa no sirve como prueba del transcript.

## Límites y pendientes

Boddy 1.0 nombra la aplicación, no un modelo entrenado desde cero. La inferencia
actual usa llama.cpp/Vulkan, no WGPU. Falta motor propio, corpus versionado,
embeddings, ejecución Python, sandbox del SO y sesiones completas reanudables.
Existe un primer investigador de lectura, sin planes abiertos ni ejecución.
El chat genera texto: la casilla de mapa aporta cifras del proyecto
desde el host, sin leer fuentes completas ni ejecutar propuestas del modelo.
El historial es limitado y se recorta por turnos. La respuesta se muestra completa
al terminar, sin streaming. Cancelar cierra el cliente; el servidor puede tardar en
liberar el trabajo. El modelo sigue cargado hasta tools/Stop-Model.ps1.

El acceso loopback se usa para inferencia autorizada por el dueño y no otorga al
modelo una herramienta de red general. La política inicial del núcleo conserva
la denegación de red y ejecución para tareas. Los módulos comparten proceso cuando
se enlazan; separar crates no constituye una frontera de seguridad del SO.

## Continuar

Las métricas y pruebas de recorte están en docs/RECURSOS.md. Puede repetirse la
comprobación real con tools/Test-ChatBudget.ps1 y el motor ya iniciado.
El agente de lectura y sus límites están en docs/AGENTE-LECTURA.md; se comprueba
con tools/Test-Research.ps1. El registro persiste evidencia, no reanuda tareas solo.

Leer super-grafo/MAPA.md y docs/PLAN.md. Mantener explícito el motor real, la
procedencia de los pesos y el alcance de cada comprobación. La siguiente mejora
puede conectar recuperación documental al contexto sin conceder ejecución de código.

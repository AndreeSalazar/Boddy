# Estado vigente: Boddy 1.0

Hardware observado: RTX 3060, 12288 MiB, controlador NVIDIA 610.74; RAM comunicada:
16 GB. Los commits 091dc56 y c9aa594 conservan las etapas iniciales del proyecto.

## Disponible

- Chat Windows sencillo: historial reciente, Enter para enviar, Shift+Enter para
  nueva línea, nuevo chat y cancelación de la espera.
- Diecinueve crates con dependencias acíclicas. Contratos, misión, política y controlador
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
- SUPER GRAFO: mapa, notas, posiciones y 27 nodos con estados explícitos.
- Contadores visibles de entrada, salida, caché y acumulados, velocidad, recorte
  e instantánea de VRAM del dispositivo al responder. Conteo previo por tokenizer
  del motor, con eliminación de pares antiguos para respetar el presupuesto.
- Casilla opcional Mapa de Boddy: el host consulta un resumen del proyecto y solo
  entrega cifras seleccionadas; no permite al modelo ejecutar herramientas.
- Modo Investigar proyecto: conecta misión, estados y cupo del núcleo a búsquedas
  y lecturas decididas por el modelo dentro de un flujo acotado. Hasta tres fuentes;
  informe con líneas/citas comprobadas y SHA256 estable. Registro en data/tasks,
  excluido de Git. Revisión semántica pendiente incluso cuando termina Completed.
- El investigador resuelve nombres de componentes, usa anclas de cita preparadas
  por el host y permite una corrección. Guarda todos los intentos y rechazos.
  Se excluyen cabeceras de función como evidencia de comportamiento y se detecta
  un caso léxico acotado de permisos afirmados contra inicializadores false.
- Ventana con Conversar, Programar, Mi PC, Evidencia, Laboratorio, Atencion y Rust. Explorador acotado de
  fuentes (incluye window como texto), fragmentos y apertura manual en editor.
  Acciones fijas para Explorer, SUPER GRAFO y Administrador de tareas, con log local.
  Vista de tareas guardadas y consulta manual de GPU, independiente del modelo.
- Boddy Tiny 1.0: MLP propia de 27.776 parámetros, ocho caracteres de contexto,
  capa tanh de 32 unidades y 96 salidas ASCII. Inicialización aleatoria; sin pesos
  Qwen ni profesor externo. Corpus educativo propio versionado, SGD por WGPU y
  referencia CPU no_std. Laboratorio muestra métricas y guarda checkpoints.
- Selector Qwen/Tiny en Conversar y biblioteca de versiones en Laboratorio.
  Continuación de entrenamiento con estado de muestreo, pasos y padre; modelos
  antiguos conservan solo pesos y se identifican como tales. Generación Tiny
  CPU sin historial, herramientas ni investigación. Lista de modelos acotada y
  validación de pesos/arquitectura antes de cargar. Control de frases no entrenadas
  informa pérdida y acierto del siguiente carácter; sin aprobación automática.
- Checkpoint propio v3: JSON compacto, pesos f32 little endian con SHA256 y
  reutilización de archivos idénticos entre versiones. Mantiene carga v1/v2.
  Ventana muestra bytes JSON, pesos y escritura nueva. No elimina versiones
  previas; el autoaprendizaje general sigue pendiente.
- Boddy Attention 0.1: decoder propio de un bloque, embeddings 16D, una cabeza
  causal, RMSNorm y FFN tanh 16/32/16. 10.544 parámetros aleatorios; UTF-8 por
  bytes, contexto 16 bytes. Entrena por intérprete escalar WGPU y genera por CPU.
  Panel separado para crear, continuar, consultar y generar; formato independiente
  de Tiny. No importa pesos Qwen y todavía repite espacios en la muestra inicial.
- Escuela Rust: perfil rust-v1 para Attention y controlador de seis ejercicios
  sobre nueve expresiones conocidas. Ranking neuronal, propuesta, verificación de
  cinco casos y hasta tres intentos. Registra primera elección y búsqueda aparte;
  guarda propuestas en data/rust-school sin compilar ni modificar proyectos.
  Perfiles nuevos conservan al padre, con transferencia explícita de pesos cuando
  cambia el corpus. Ver docs/ESCUELA-RUST.md.

## Verificado en esta entrega

- cargo test --workspace --offline: 50 pruebas aprobadas.
- cargo clippy --workspace --all-targets --offline -- -D warnings: aprobado.
- cargo check -p boddy-core --target wasm32-unknown-unknown --offline: aprobado.
- cargo check -p boddy-neural --target wasm32-unknown-unknown --offline: aprobado.
- cargo check -p boddy-transformer --target wasm32-unknown-unknown --offline: aprobado.
- cargo check -p boddy-rust-tasks --target wasm32-unknown-unknown --offline: aprobado.
- Escuela Rust real: 300 pasos sobre un Attention general propio. Pérdida Rust
  entrenamiento 4,4870 -> 3,8320; validación 4,8796 -> 4,1192. Primera elección
  0/6 -> 1/6; búsqueda de hasta tres intentos 2/6 -> 2/6. Cuatro pendientes,
  estado Stopped; propuestas de suma y resta pasaron solo sus casos finitos.
  Perfil y ciclo también funcionaron desde la ventana. Sin evidencia de Rust
  general ni ejecución autónoma de proyectos. evals/rust-school-v1-rtx3060.json.
- Attention real en RTX 3060: 300 pasos, pérdida de entrenamiento 5,5408 -> 3,9268
  y validación 5,5679 -> 4,1217. Error máximo de gradientes GPU/CPU 4,77e-7.
  200 pasos frente a 100 + 100 produjeron los mismos pesos y muestreador; padre
  intacto y prefijo Unicode conservado. JSON inicial 508 bytes, pesos 42.176 bytes;
  buffers GPU declarados 1.081.976 bytes, sin incluir driver o pico real.
  Pasó entrenamiento 300, continuación a 400 y generación Unicode desde ventana.
  No es evidencia de chat útil o precisión de respuestas. Ver docs/ATTENTION.md.
- Entrenamiento real Tiny en RTX 3060 por WGPU/Vulkan: 300 pasos redujeron pérdida
  de entrenamiento 4,5659 -> 2,5693 y validación 4,5703 -> 2,5811. Desde el botón
  de ventana, 1.000 pasos: 1,8081 y 2,1641. Error máximo absoluto de gradientes
  CPU/GPU 7,45e-9; pérdida 4,77e-7; SGD 7,45e-9. Muestra restaurada reproducible.
  239.904 bytes de buffers declarados; no es VRAM total ni pico. Datos y
  validación pequeños/relacionados: no demuestran capacidad de asistente.
- Continuación real en RTX 3060: 600 pasos frente a 300 + 300 reanudados dieron
  pesos iguales, mismo muestreador y padre intacto. Control no entrenado: acierto
  del siguiente carácter 24,6 % -> 28,9 % en la segunda etapa. Evidencia en
  evals/tiny-continuation-v1-rtx3060.json. Prueba nativa de ventana: crear 1.000
  pasos, continuar hasta 1.300, seleccionar Tiny para generación y volver a Qwen;
  actualizar versiones conserva el historial. No es inspección visual del diseño.
- Formato v3 verificado con restauración bit a bit, misma continuación 300 + 300
  frente a 600 y mismo texto. Comparación de pesos iguales: 576.096 bytes de
  JSON v2 frente a 576 de JSON v3 + 111.104 de binario. Archivo compartido para
  las dos trayectorias, sin copiar otra vez sus pesos. Pruebas rechazan cambios
  de bytes, truncamiento, referencias fuera de alcance y NaN. Evidencia en
  evals/tiny-storage-v3-rtx3060.json; cifras de contenido en disco, no RAM/VRAM.
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
- Evaluación de lectura v1: línea base 1/3; la ronda final del protocolo de anclas,
  alias de componentes y revisión obtuvo 3/3 en citas y fuente esperada. El filtro
  false rechazó una contradicción visible y el modelo la retiró. No se puntuó la
  corrección semántica completa; casos conocidos y solo tres objetivos.
- VRAM total observada tras cargar y generar: aproximadamente 5,5–5,6 GiB,
  incluyendo el uso previo del escritorio. No es consumo aislado del proceso,
  una medición del pico ni un benchmark de calidad o velocidad general.
- Lanzador nativo compilado sin consola; proceso Windows con título Boddy 1.0.
  La herramienta visual no expuso sus ventanas PowerShell; la verificación de
  interfaz se limita al test nativo de controles/eventos. DrawToBitmap no captura
  correctamente RichTextBox y su vista previa no sirve como prueba del transcript.

## Límites y pendientes

Boddy 1.0 nombra la aplicación. Su laboratorio Boddy Tiny sí entrena desde cero;
el chat sigue usando Qwen por llama.cpp/Vulkan. Tiny no sustituye al asistente,
reanuda entrenamientos y permite elegir checkpoints para continuar texto ASCII,
pero no aprende conversaciones. Falta inferencia propia
de un asistente general, corpus documental, recuperación semántica, ejecución Python,
sandbox del SO y sesiones completas reanudables. Ver MODELO-PROPIO.md.
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

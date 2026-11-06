# Primer agente de lectura — 6 de noviembre de 2026

La casilla **Investigar proyecto (lectura)** de la ventana ejecuta una tarea
independiente del historial del chat, sobre Boddy. También existe el comando:

```powershell
'{"objective":"Explica como budget.rs limita el contexto de tokens."}' |
    .\target\debug\boddy-cli.exe investigar
```

## Ciclo implementado

1. El host crea una misión y una tarea con la política local del núcleo no_std.
2. Consulta el resumen de SUPER GRAFO.
3. El modelo propone una o dos búsquedas de nombres de archivo.
4. El host realiza esas búsquedas y filtra los resultados por el alcance permitido.
5. El modelo selecciona de uno a tres índices del listado, nunca rutas arbitrarias.
6. El lector obtiene fragmentos acotados, números de línea y SHA256 de las fuentes.
7. El modelo redacta hallazgos estructurados con citas textuales.
8. El host comprueba que cada cita exista en las líneas indicadas y vuelve a leer
   las fuentes para comprobar que sus hashes no cambiaron.
9. Solo entonces completa el contrato de citas de la tarea y entrega el informe.

No se verifica automáticamente la verdad de cada conclusión. `Completed` significa
que el informe cumple el contrato técnico de citas; `semantic_review` permanece
`pending`. Una fuente citada puede ser insuficiente o estar equivocada. La lectura
de un fragmento tampoco equivale a una revisión completa del archivo o del proyecto.

## Fronteras y presupuestos

- Tres llamadas al modelo como máximo: búsquedas, selección e informe.
- Hasta seis herramientas del núcleo: un resumen, dos búsquedas y tres lecturas.
- El verificador hace además hasta tres relecturas del host; se registran aparte.
- Máximo de 16 candidatos, tres fuentes y cinco hallazgos validados.
- Por fuente: archivo UTF8 de hasta 256 KiB, fragmento de hasta 35 líneas y 1600
  caracteres de texto. Se selecciona el primer acierto de los términos con contexto.
- Objetivo de hasta 800 caracteres; cada llamada usa el presupuesto del adaptador
  (4096 tokens de contexto, 512 de salida). Si no cabe, se detiene con un motivo.
- Solo fuentes `.rs`, `.md`, `.toml`, `.ps1` o `.vbs` dentro de `crates/`, `docs/`,
  `window/`, o los archivos
  raíz `README.md` y `Cargo.toml`. No se ofrecen `runtime`, pesos, Git ni credenciales.
- Se rechazan rutas absolutas, componentes ocultos, escapes, enlaces y puntos de
  reanálisis de Windows. Se comprueba también la ruta canónica bajo el proyecto.

Estos controles pertenecen al host y no son un sandbox del SO ni eliminan todas las
carreras de acceso a archivos. El modelo no recibe shell, escritura, red general,
instalación o compilación. La única red del flujo es la inferencia loopback existente.
Los archivos leídos se tratan como datos, no como autorización o instrucciones.

## Registro y fallos

`data/tasks/read-<id>.json` conserva objetivo, eventos, fuentes, fragmentos, hashes,
uso de tokens, informe y estado. Está excluido de Git porque puede contener datos
del dueño. El agente escribe ese registro; la restricción de lectura se refiere a
las acciones sobre las fuentes del proyecto, que no se modifican.

Si el modelo devuelve un índice o una cita inválidos, faltan archivos o el motor
no está disponible, el registro queda `Stopped` y no se muestra un informe aprobado.
No hay reintentos infinitos ni reparación oculta de citas. El registro permite
inspeccionar lo sucedido; aún no existe reanudación automática desde él.

La ventana espera hasta 420 segundos en este modo. Cancelar termina el cliente;
la liberación del servidor es de mejor esfuerzo. Una cancelación o caída abrupta
puede dejar el último checkpoint activo y un temporal: no equivale a una tarea
completa. No se promete recuperación transaccional de una tarea interrumpida.

## Evidencia

Se ejecutó un objetivo sobre `budget.rs` con el modelo local. Seleccionó la fuente,
produjo citas de las constantes y una función, y el host verificó líneas y SHA256.
La tarea terminó en `Completed` con tres llamadas al modelo. También pasó desde
la ventana sin modificar el historial de chat. El caso sin motor dejó `Stopped`.

Para repetir con el motor iniciado y el host compilado:

```powershell
.\tools\Test-Research.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -STA -File .\window\Boddy.ps1 -ResearchSmokeTest
```

La autonomía actual es una secuencia de investigación acotada con decisiones del
modelo sobre búsquedas y archivos. Planes abiertos, reanudación, parches, pruebas
y ejecución aislada siguen pendientes.

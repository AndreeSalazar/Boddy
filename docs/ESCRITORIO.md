# Espacios de trabajo Windows

La ventana reúne cuatro espacios sin mezclar reglas del núcleo con controles UI.

| Espacio | Acciones implementadas |
|---|---|
| Conversar | Chat, presupuestos, métricas e investigación de lectura |
| Programar | Listar fuentes de Boddy, ver fragmento y SHA256, abrir una fuente en Bloc de notas, investigar selección |
| Mi PC | Abrir proyecto en Explorer, SUPER GRAFO, Administrador de tareas, carpeta de registros y consultar GPU |
| Evidencia | Listar hasta veinte tareas recientes y leer el registro seleccionado |

En Programar se puede elegir explicar, revisar riesgos o proponer un plan de mejora.
El botón de investigación transforma esa selección en un objetivo y usa el mismo
agente de lectura. No aplica un parche. El editor se abre para que el dueño edite;
las validaciones y permisos del modelo no cambian al abrirlo.

## Frontera de control

`boddy-desktop` enumera las aperturas permitidas y construye argumentos separados,
sin shell. Los botones son acciones explícitas del dueño y no herramientas ofrecidas
al modelo. Las propuestas del modelo no pueden activar esta API desde el chat.
La CLI local sí puede invocarla: esto no es una frontera de autenticación del SO.

Las aperturas se registran en `runtime/desktop-actions.jsonl` con acción y resultado
del lanzamiento. Un PID indica que se inició el proceso, no que verificamos su
ventana o todas sus operaciones posteriores. El log es local y editable, no una
auditoría criptográfica. No se solicita elevación de privilegios automáticamente.

El proyecto de trabajo en esta entrega es Boddy. No existe selector de acceso
irrestricto a discos, registro o credenciales, ni ejecución arbitraria de comandos.
Los controles de archivos usan el lector acotado del proyecto. Los registros de
tareas se identifican por `read-<numero>`, nunca por rutas libres.

Es un inicio de control manual del escritorio, no control completo del PC por
la IA. Acciones automáticas con escritura, procesos, teclado/ratón y múltiples
proyectos requieren contratos y comprobaciones adicionales antes de conectarse.

## Comprobaciones

La prueba nativa de ventana cargó cuatro pestañas, la lista de fuentes y el
fragmento de budget.rs, los registros y una respuesta de recursos. Comprueba también
que la caja y las métricas del chat permanezcan dentro del área visible.
Las aperturas de aplicaciones se probaron como planes de argumentos y con rechazo
de acciones no registradas; no se automatizaron esas aplicaciones durante la prueba.

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -STA -File .\window\Boddy.ps1 -WorkbenchSmokeTest
'{"action":"open_project","preview":true}' | .\target\debug\boddy-cli.exe pc
```

El host puede utilizar Programar, Mi PC y Evidencia sin tener el modelo cargado.
Si el arranque del motor falla, el lanzador mantiene accesibles los controles locales.
La GPU se consulta al pulsar el botón: es una instantánea del dispositivo completo,
incluyendo otras aplicaciones, no uso exclusivo de Boddy ni consumo pico.

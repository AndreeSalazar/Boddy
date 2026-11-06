# Arquitectura de Boddy

## Capas y responsabilidad

```text
Petición del dueño
       |
Host Rust (std): sesión, contexto, adaptadores y verificación
       |                   |                     |
boddy-core (no_std)    SUPER GRAFO          Biblioteca documental
misión, estados,      consultas acotadas    fuentes y fragmentos
capacidades y cupo
       |
Inferencia sustituible [pendiente]
       |
Propuesta de acción -> validación del host -> ejecutor aislado [pendiente]
                                            |
                                       resultado y pruebas
```

`boddy-core` no interpreta lenguaje natural. Controla una tarea sin archivos,
procesos, red, heap ni dependencia del sistema. Sus datos son referencias prestadas
y un máximo de ocho referencias de evidencia. Las referencias no contienen los
documentos ni certifican su autenticidad.

Estados actuales: `Created -> Researching -> Planned -> Verifying -> Completed`.
Una revisión vuelve de `Verifying` a `Planned` e invalida la aprobación previa.
Una tarea activa puede detenerse. Un resultado fallido o una verificación ausente
impiden marcarla completa. El host debe evaluar el criterio de aceptación antes
de informar que una verificación pasó; el núcleo no puede comprobarlo por sí solo.

El cupo de llamadas se consume antes de ejecutar la herramienta y no se reinicia
al revisar. Es por instancia de tarea en memoria: todavía no sobrevive al reinicio.
La política inicial permite lectura y propuesta; deniega código y red. No mide
tokens ni tiempo de inferencia. Los límites de tiempo/salida de SUPER GRAFO viven
en el adaptador del host, no en el núcleo.

## Contratos a implementar después

| Contrato | Entrada | Salida comprobable |
|---|---|---|
| Modelo | Contexto seleccionado y esquemas de herramientas | Texto o propuesta estructurada; nunca permisos nuevos |
| Contexto | Consulta, proyecto y presupuesto | Fragmentos con ruta/URL, versión y procedencia |
| Ejecutor | Acción validada, recursos y permisos | Código de salida, stdout/stderr acotados y artefactos |
| Verificador | Criterio de aceptación y artefactos | Resultado, alcance de la prueba y referencia de evidencia |
| Sesión | Eventos y referencias | Registro persistente y resumen para continuar |

Una interfaz de modelo permitirá cambiar pesos o backend sin reescribir la misión.
Antes de declararla lista se probará que las operaciones y la cuantización elegidas
produzcan resultados correctos. Los shaders y la inferencia WGPU aún no existen.

## Fronteras de confianza

El host, su configuración y sus verificadores son de confianza. El modelo,
documentos recuperados y código de proyectos pueden aportar datos o propuestas;
no pueden modificar capacidades. Una instrucción dentro de un documento se trata
como contenido de la fuente y no como una orden del dueño.

El aislamiento real debe restringir archivos, procesos y red desde el SO. También
debe considerar controladores, acceso a GPU y recursos compartidos. `no_std`, Rust,
validar argumentos o lanzar un subproceso no proporcionan aislamiento completo.
La primera versión no ofrece ejecutar código arbitrario.

## Memoria y continuidad

Ahora: documentación y mapa versionables del proyecto. Después: registro de eventos
persistente con identidad de tarea, razón, plan, llamadas, resultados y revisiones.
El resumen para el modelo será una vista de ese registro; no su única copia.
La memoria de proyectos se mantendrá separada de datos privados y pesos del modelo.

Arranque bare metal completo, controladores propios y un SO nuevo quedan como un
alcance distinto. La base actual inicia el núcleo `no_std` desde el primer día y
usa Windows como host para verificar progreso sobre el hardware disponible.

# Por qué existe Boddy

Porque investigar, programar y comprobar un proyecto requiere sostener un objetivo,
encontrar información fiable y convertirla en acciones que realmente funcionan.
El dueño necesita una herramienta personal que mantenga esa continuidad y permita
entender qué hizo, con qué información y con qué resultado.

## Para qué

1. Investigar preguntas técnicas con fuentes identificables y versiones pertinentes.
2. Entender proyectos mediante SUPER GRAFO y leer después los archivos necesarios.
3. Transformar una petición en un plan pequeño con una condición de aceptación.
4. Preparar código y, cuando exista el ejecutor aislado, probarlo y corregirlo.
5. Retomar el trabajo conservando decisiones, evidencia, pendientes y errores.

## Cómo reconocer una tarea bien hecha

Cada tarea tiene una razón, un resultado buscado y un criterio de aceptación.
Una respuesta técnica debe distinguir lo observado de lo inferido. Un cambio de
código debe acompañarse de una comprobación pertinente al comportamiento buscado.
Si faltan datos o falla una prueba, el resultado debe mostrarlo y conservar el
pendiente; el texto del modelo no puede convertirse por sí solo en aprobación.

La primera tarea completa futura será: entender un repositorio pequeño, localizar
un fallo, proponer un parche, comprobarlo en un entorno aislado y entregar el cambio
con su evidencia. Primero debe existir y comprobarse la infraestructura necesaria.

## Papel de cada tecnología

- Rust: implementar el sistema y sus contratos.
- `no_std`: mantener un núcleo independiente de los servicios del sistema operativo.
- Modelo neuronal: interpretar peticiones y generar propuestas.
- SUPER GRAFO: localizar contexto y relaciones del proyecto.
- Documentación: aportar información consultable y trazable.
- `wgpu`: candidato para acelerar cómputo del modelo en la GPU.
- Sandbox del sistema: restringir la ejecución de herramientas.

## Alcance personal inicial

Investigación técnica y programación en los proyectos del dueño. El conocimiento
se amplía por fuentes y tareas concretas, con prioridades medibles. Inspirarse en
publicaciones comerciales permite aprender patrones públicos; no proporciona sus
pesos, datos privados ni garantiza alcanzar su capacidad.

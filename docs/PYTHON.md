# Convertir conocimiento de Python en utilidad

"Todos los datos de Python" se concreta en tres trabajos diferentes: consultar
documentación, disponer de herramientas y portar componentes elegidos. Cada uno
tiene entradas, licencias, versiones y pruebas distintas. No hay una conversión
automática que transforme todo el ecosistema Python en Rust `no_std`.

## De referencia a capacidad

| Fuente oficial | Utilidad buscada | Implementación prevista |
|---|---|---|
| [Tutorial](https://docs.python.org/3/tutorial/) | Explicar y escribir Python según su versión | Corpus consultable con ejemplos y evaluaciones |
| [ast](https://docs.python.org/3/library/ast.html) | Identificar funciones e imports sin ejecutar un programa | Herramienta Python externa versionada, o parser Rust que se evalúe por separado |
| [subprocess](https://docs.python.org/3/library/subprocess.html) | Ejecutar herramientas y obtener resultados | Contrato Rust del host con argumentos separados, límites y entorno restringido |
| [sqlite3](https://docs.python.org/3/library/sqlite3.html) | Guardar sesiones y evidencia | Evaluar SQLite desde Rust en el host; el núcleo continúa sin base de datos |
| [json](https://docs.python.org/3/library/json.html) | Intercambiar resultados estructurados | Serialización versionada y límite de tamaño en la frontera del host |
| [pathlib](https://docs.python.org/3/library/pathlib.html) | Trabajar con rutas con semántica explícita | Tipos Rust y comprobaciones de alcance en el host; contemplar enlaces y junctions |

Las correspondencias de implementación son propuestas propias, no equivalencias
completas entre las APIs de ambos lenguajes. El primer adaptador real de procesos
es el de SUPER GRAFO; no incluye todavía un intérprete Python.

## Camino de ingestión documental

1. Elegir la versión de Python y fijar URLs o una revisión de sus fuentes.
2. Descargar solo documentación autorizada, guardando fecha, hash, origen y términos.
3. Separar texto explicativo, código y metadatos; conservar enlaces al original.
4. Partir por secciones y símbolos con límites de tamaño.
5. Probar primero un índice léxico local; añadir embeddings si mejoran la evaluación.
6. Recuperar fragmentos según la consulta, citar su versión y contrastar ejemplos.

Hoy solo están registrados los enlaces: `metadata_only` significa que **no se han
descargado ni indexado sus páginas completas**. Las URLs `/3/` son móviles; antes
de la ingestión deben fijarse una versión y un snapshot. Consultar una página no
equivale a ingerir un corpus o entrenar un modelo con ella.

## Herramientas y portabilidad

Para cada herramienta se define entrada, salida, permisos, efectos, timeout y
comprobación. Si depende de un intérprete, librerías nativas o red, esas dependencias
se declaran en el host. Un ejecutor Python futuro se probará dentro del sandbox
antes de ofrecerlo al agente.

Un port a Rust se decide por componente: primero su comportamiento y ejemplos,
después su interfaz y finalmente pruebas comparativas. Cambiar de lenguaje no
preserva automáticamente tipos dinámicos, excepciones, precisión numérica o APIs.

[Los términos de Python](https://docs.python.org/3/license.html) son una fuente
de referencia; cada paquete de terceros mantiene sus propios términos. Este inicio
guarda enlaces y resúmenes propios. Cualquier copia posterior conservará procedencia
y los avisos que correspondan al material concreto.

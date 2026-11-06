# Biblioteca: registro, contexto y memoria documental

La biblioteca permite localizar documentos sin enviarlos todos a un modelo.
`boddy-library` implementa el servicio en Rust del host; no depende de Python,
GPU, inferencia ni cambios del nucleo no_std. La ventana consume sus comandos.

## Flujo real

1. **Categorias y busqueda:** lee categorias.json y registro.json. Compara titulo,
   etiquetas y resumen; no abre cuerpos documentales. Ranking lexico, no semantico.
2. **Contexto:** lee hasta tres fichas candidatas de Markdown y selecciona una
   seccion pertinente por documento. Incluye ID, lineas y SHA256 en el contexto.
3. **Recordar:** persiste pregunta, presupuesto y referencias ID/ruta/hash. No
   duplica el texto del documento. La lista Recuerdos recupera esos registros.
4. **Releer:** resuelve los mismos IDs en el registro actual, abre de nuevo sus
   documentos y compara hashes/rutas. Puede usar otra pregunta para elegir otra
   seccion de las mismas referencias. Buscar otra vez sirve para cambiar de area.
5. **Exportar:** escribe un paquete JSON y un documento Markdown con el contexto
   seleccionado y su procedencia. No exporta ni inyecta toda la biblioteca.

Las fuentes son notas propias con enlaces oficiales y protocolos del proyecto.
Un hash detecta cambios del contenido; no demuestra que una afirmacion sea cierta.
Los documentos y enlaces son datos, no instrucciones para ejecutar herramientas.

La biblioteca aun no se inyecta automaticamente al chat ni a los modelos propios.
Se puede inspeccionar y exportar el paquete para integraciones futuras. Attention
solo tiene 16 bytes de contexto; este trabajo no le proporciona comprension de
documentos largos, nuevos pesos ni aprendizaje automatico.

## Uso

En **Biblioteca**, cargar **Categorias**, buscar una ficha y pulsar **Contexto**.
Elegir 256, 512, 1024, 2048 o 4096 bytes. **Ver fragmento** consulta una ficha
seleccionada; no muestra necesariamente todo el archivo. **Recordar IDs** guarda
la consulta. **Recuerdos** carga registros previos y **Releer** usa la pregunta
actual sobre sus referencias. El boton Exportar prepara la consulta actual;
la API tambien permite exportar un recuerdo por ID.

**Idiomas IANA** busca nombres o codigos como es, en, fr o ja. Informa si hay solo
catalogo o esquemas locales. Nombres originales del registro, principalmente en
ingles; se admiten algunos alias de consulta en espanol. No descarga cursos.

```powershell
'{"action":"categories"}' | .\target\debug\boddy-cli.exe biblioteca
'{"action":"search","query":"rust prestamos"}' | .\target\debug\boddy-cli.exe biblioteca
'{"action":"context","query":"rust prestamos","budget_bytes":512}' | .\target\debug\boddy-cli.exe biblioteca
'{"action":"remember","query":"rust prestamos","budget_bytes":512}' | .\target\debug\boddy-cli.exe biblioteca
'{"action":"memories"}' | .\target\debug\boddy-cli.exe biblioteca
# Sustituir mem-<id> por un recuerdo real:
'{"action":"recall","memory_id":"mem-<id>","query":"referencias mutables"}' | .\target\debug\boddy-cli.exe biblioteca
'{"action":"export","memory_id":"mem-<id>"}' | .\target\debug\boddy-cli.exe biblioteca
'{"action":"languages","query":"fr"}' | .\target\debug\boddy-cli.exe biblioteca
```

## Presupuesto y persistencia

El limite cuenta bytes UTF-8 del campo context, incluidas sus cabeceras de cita.
No cuenta todos los mensajes, metadatos del JSON, transporte ni tokens reales.
`token_count` queda null: cada modelo debe contar con su tokenizer. No se afirma
un ahorro porcentual de tokens, RAM o VRAM sin una medicion comparativa adecuada.

La prueba observada uso un limite de 512 bytes: busqueda con cero documentos
abiertos; contexto de 375 bytes desde una ficha de 912 bytes; recuerdo de 336
bytes y paquete JSON exportado de 1.406 bytes. Evidencia en evals/library-v1.json.

Recuerdos en data/library/memories y exportaciones en data/library/exports,
ignorados por Git. Datos fuente y registro de categorias/fichas en biblioteca.
Los recuerdos guardan referencias; si desaparece un ID o archivo, la relectura
falla explicitamente. IDs estables permiten mover documentos actualizando rutas.

Limites iniciales: 16 categorias, 256 fichas, registro 128 KiB, categorias 16 KiB,
Markdown 16 KiB, contexto 256..4096 bytes, tres documentos por paquete, recuerdos
8 KiB, listado de hasta 64 entre 512 entradas. Para una biblioteca mucho mayor
habra que particionar registros por escuela; esa extension sigue pendiente.
Las rutas deben estar registradas dentro de biblioteca, ser Markdown y carecer
de travesias/enlaces/puntos de reanalisis. No es un sandbox del sistema operativo.

## Idiomas y comprobaciones

La instantanea IANA fecha 2026-09-17 contiene 8.276 registros de tipo language,
incluidos alias obsoletos y rangos. El TSV compacto ocupa 153.050 bytes. La consulta
recorre lineas con memoria acotada, devuelve hasta doce coincidencias y comprueba
el hash/numero de registros contra procedencia.json. No entrega todo el catalogo
como contexto. Espanol e ingles tienen solo esquemas; los demas no tienen cursos.

Seis pruebas de biblioteca: busqueda solo de metadatos, presupuesto y Unicode,
relectura de contenido modificado, rutas/presupuesto invalidos, exportacion real
y distincion entre catalogo y escuela. El test nativo de ventana comprobo categorias,
busqueda, contexto, memoria, relectura, exportacion e idiomas sin alterar el chat.
Verificacion funcional; no es inspeccion visual del diseno. Repetir con
tools/Test-Library.ps1 o Boddy.ps1 -LibrarySmokeTest.

Fuentes: [registro oficial IANA](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry)
y capitulos oficiales de Rust enlazados en cada ficha. Las notas son resúmenes
introductorios propios, no copias completas ni una ingesta de toda la web.

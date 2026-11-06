# Biblioteca de Boddy

Documentos propios breves, referencias a fuentes y un registro consultable por
Rust. Es memoria documental; guardar un archivo no entrena las neuronas del modelo.

| Carpeta | Categoria | Contenido actual |
|---|---|---|
| 01-forja | La Forja | Ocho fichas iniciales de Rust: mapa, propiedad, prestamos, lifetimes, traits, errores, concurrencia y pruebas |
| 02-babel | Babel | Catalogo IANA, plantilla y esquemas de espanol/ingles; cursos pendientes |
| 03-agora | Agora | Protocolo inicial de evidencia, hipotesis y contradicciones |
| 04-atelier | Atelier | Protocolo inicial de formulacion de problemas y alternativas |
| 05-bitacora | Bitacora | Protocolo de referencias, contexto y relectura |

`categorias.json` define las categorias. `registro.json` define 14 fichas con ID,
resumen, etiquetas, fuentes, ruta, estado y relaciones `related`. Esas relaciones
son el grafo documental: pueden ser bidireccionales; no se recorren automaticamente.
SUPER GRAFO registra ademas las categorias como componentes de datos separados.

Para ampliar: crear un Markdown en su categoria y registrar su ficha. Mantener
IDs estables, documentos pequenos y procedencia. Una ruta de estudio no es un
curso completo; las fichas de Rust son introductorias y no cubren todo el lenguaje.

Babel contiene todos los registros `Type: language` de la instantanea oficial
IANA con fecha 2026-09-17: 8.276 registros en 153.050 bytes. Incluye alias
obsoletos, macrolenguajes, colecciones y un rango privado; no son necesariamente
8.276 idiomas distintos. Registrar un codigo no significa tener su escuela ni
que Boddy conozca ese idioma. Se crean carpetas solo cuando hay contenido.

Procedencia y hashes en `02-babel/procedencia.json`. La cache original esta fuera
de Git en data/library/cache. Build-LanguageRegistry.cjs convierte una descarga
oficial ya realizada; no abre red. La consulta de biblioteca tampoco usa red.

Uso, presupuestos y limites en [docs/BIBLIOTECA.md](../docs/BIBLIOTECA.md).

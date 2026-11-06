# Mapa de continuidad de Boddy

Generado por `tools/Update-SuperGrafo.ps1` a partir de una consulta real de SUPER
GRAFO y la descripción explícita de las capas de Boddy. No es una exportación
de la herramienta `guardar`: el ejecutable disponible aún no la implementa.

## Empezar aquí

1. Leer `docs/ESTADO.md`, `docs/PROPOSITO.md` y `docs/PLAN.md`.
2. Ejecutar `cargo run -p boddy-cli --offline -- grafo resumen` para datos actuales.
3. Consultar solo las partes necesarias y contrastar dependencias con Cargo.

## Capas

| Ruta | Papel | Depende de |
|---|---|---|
| `crates/boddy-core` | Misión, permisos, cupo, estados y verificación | `core`, sin paquetes externos |
| `crates/boddy-cli` | Host, catálogo, demo e integración SUPER GRAFO | `boddy-core` y `std` |
| `knowledge/sources.tsv` | Veinte fuentes: enlaces y metadatos | Fuentes oficiales consultadas |
| `docs` | Propósito, investigación, diseño, plan y estado | Decisiones y evidencia del proyecto |
| `tools/Update-SuperGrafo.ps1` | Actualizar este mapa y el resumen | Ejecutable externo de SUPER GRAFO |

## Última instantánea

Archivos indexados: 32. Relaciones léxicas: 7. Ciclos reportados:
0. Fuente exacta de estas cifras: `super-grafo/resumen.json`.
Son una instantánea: generar este mapa agrega o modifica archivos y no produce
una fotografía inmutable de sí mismo. Consultar de nuevo para el estado actual.

El análisis es léxico y no equivale a una compilación o auditoría. Sus sospechas
no son defectos confirmados. Las comprobaciones de Rust están en `docs/ESTADO.md`.

## Lo que sigue

Corpus documental versionado, comparación de modelos locales e inferencia real;
después ejecutor aislado y ciclo del agente. Hoy no hay LLM ni sandbox operativo.
Las notas de cada nodo viven en `super-grafo/NOTAS.md`.

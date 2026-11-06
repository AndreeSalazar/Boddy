# Boddy

IA operativa personal de Eddi Salazar: investigar, entender proyectos, preparar
cambios y comprobar sus resultados, con Rust como base y SUPER GRAFO como mapa.

La primera entrega fija el **por qué, para qué y cómo**, registra fuentes
primarias y construye el controlador de tareas. Todavía no genera respuestas con
un modelo, no ejecuta Python ni tiene un sandbox del sistema operativo.

- [Propósito](docs/PROPOSITO.md): qué problema resuelve y qué cuenta como éxito.
- [Investigación](docs/INVESTIGACION.md): publicaciones consultadas y decisiones propias.
- [Arquitectura](docs/ARQUITECTURA.md): núcleo `no_std`, host, modelo y aislamiento.
- [Python como utilidad](docs/PYTHON.md): documentos, herramientas y portabilidad.
- [Plan](docs/PLAN.md): entregas y condiciones de aceptación.
- [Estado](docs/ESTADO.md): lo comprobado y lo pendiente.
- [Mapa](super-grafo/MAPA.md): orientación para retomar el trabajo.

## Probar la base

Desde esta carpeta, con Rust y el linker de la plataforma instalados:

```powershell
cargo test --workspace --offline
cargo check -p boddy-core --offline
cargo run -p boddy-cli --offline -- proposito
cargo run -p boddy-cli --offline -- fuentes python
cargo run -p boddy-cli --offline -- demo
cargo run -p boddy-cli --offline -- grafo resumen
```

Las dos crates no dependen de paquetes externos. `boddy-core` usa exclusivamente
`core`, sin `std`, `alloc`, heap, I/O ni código `unsafe`.
La CLI usa `std` para acceder al sistema anfitrión.

`fuentes` busca títulos y metadatos del catálogo local; **no busca dentro de
documentos completos**. La demo muestra el controlador y sus rechazos de permisos;
es determinista, sin inferencia ni tareas reales resueltas por una IA.

## SUPER GRAFO

`grafo` invoca el ejecutable existente en la carpeta hermana
`SUPER GRAFO/ultra-carpetas/target/release/ultra-carpetas.exe`. Para otra instalación:

```powershell
$env:BODDY_SG_BIN = 'C:\ruta\ultra-carpetas.exe'
cargo run -p boddy-cli --offline -- grafo resumen
```

La integración permite `resumen`, `fallos` y `ciclos` sobre Boddy; envía argumentos
por separado, sin shell. Limita stdout y stderr a 64 KiB cada uno y espera hasta
30 segundos. El analizador puede escribir su caché de metadatos: esto no equivale
a un sistema de archivos montado como solo lectura. SUPER GRAFO sigue siendo un
programa externo; no se copió ni enlazó su código en el núcleo.

Para actualizar el mapa de continuidad y su instantánea de cifras:

```powershell
.\tools\Update-SuperGrafo.ps1
```

El script de mantenimiento usa el resumen de SUPER GRAFO y escribe únicamente
el mapa y su JSON en `super-grafo/`. Las notas se mantienen por separado.

Hardware objetivo comunicado: RTX 3060 de 12 GB y 16 GB de RAM. Aún no se han medido
inferencia, consumo de VRAM ni velocidad en ese equipo.

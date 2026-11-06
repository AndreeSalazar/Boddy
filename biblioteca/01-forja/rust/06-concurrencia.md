# Rust: concurrencia y paralelismo

## Para que existe
La concurrencia organiza trabajos que avanzan de forma independiente. El paralelismo permite ejecutar trabajos al mismo tiempo. Propiedad y tipos ayudan a detectar ciertos accesos incompatibles durante la compilacion.

## Que estudiar
Hilos, mensajes y estado compartido con sincronizacion. Elegir el modelo segun las dependencias del trabajo. Agregar hilos no garantiza acelerar una tarea ni evita por si solo bloqueos o errores de negocio.

## Comprobacion
Dibujar que datos comparte cada tarea y quien puede modificarlos. Medir el resultado y el tiempo antes de atribuir una mejora al paralelismo.

## Fuente
Nota propia introductoria.
https://doc.rust-lang.org/book/ch16-00-concurrency.html

# Rust: propiedad y memoria

## Para que existe
La propiedad u ownership relaciona cada valor con su propietario y con el momento en que sus recursos se liberan. Rust comprueba estas reglas durante la compilacion; no exige un recolector de basura para aplicar este sistema.

## Que estudiar
Movimiento de valores, copia cuando el tipo lo permite y finalizacion del propietario. Un movimiento puede impedir usar de nuevo el nombre anterior. Eso no significa que todo valor se copie ni que todas las operaciones carezcan de coste.

## Comprobacion
Examinar quien posee cada String antes y despues de pasarlo a una funcion. Distinguir transferir propiedad de prestar una referencia. Ver despues la escuela de prestamos.

## Fuente y alcance
Nota propia introductoria, no copia completa del libro ni prueba de seguridad de un programa.
https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html

# Rust: errores y recuperacion

## Para que existe
Result representa una operacion que puede terminar con un valor o un error recuperable. panic interrumpe la ejecucion normal ante una situacion que el programa no trata como un resultado recuperable.

## Que estudiar
Tipos de error, propagacion y decisiones de recuperacion. El objetivo es hacer visible el fallo y permitir una respuesta pertinente, en lugar de presentar un resultado inexistente como correcto.

## Comprobacion
Para leer un archivo, distinguir una ruta inexistente de contenido invalido. Decidir quien debe comunicar, propagar o manejar cada error. Las pruebas deben comprobar tambien esos casos.

## Fuente
Nota propia introductoria.
https://doc.rust-lang.org/book/ch09-00-error-handling.html

# Rust: duracion de referencias

## Para que existe
Los lifetimes describen relaciones entre los periodos durante los cuales las referencias son validas. Ayudan a rechazar referencias que podrian apuntar a valores que ya no existen.

## Que estudiar
Inferencia de lifetimes y anotaciones en firmas cuando hay relaciones ambiguas. Una anotacion no prolonga la vida real del dato: expresa una relacion que el compilador debe poder comprobar.

## Comprobacion
Identificar el propietario de un valor retornado por referencia y comprobar que vive lo suficiente. No intentar arreglar una referencia colgante agregando nombres de lifetime al azar.

## Fuente
Nota propia introductoria.
https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html

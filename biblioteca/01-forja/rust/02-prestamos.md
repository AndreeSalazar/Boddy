# Rust: referencias y prestamos

## Para que existe
Un prestamo permite acceder a un valor sin transferir su propiedad. Las referencias compartidas permiten lectura; una referencia mutable permite modificar y requiere acceso exclusivo mientras ese prestamo esta activo.

## Que estudiar
Relacion entre propietario y referencia, ambitos de uso y combinaciones permitidas de acceso compartido y mutable. El compilador evita combinaciones de prestamos incompatibles. Estas reglas no convierten automaticamente toda logica de una aplicacion en correcta.

## Comprobacion
Comparar una funcion que consume un String con otra que recibe una referencia. Identificar por que una modificacion compite con una referencia compartida que sigue usandose. Releer lifetimes si el problema es la validez de la referencia.

## Fuente
Nota propia introductoria.
https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html

# Rust: pruebas y evidencia

## Para que existe
Una prueba prepara condiciones, ejecuta el comportamiento bajo estudio y compara lo observado con una expectativa. Las funciones marcadas como test se integran en el ejecutor de pruebas de Rust.

## Que estudiar
Casos normales, limites y fallos relevantes. Una prueba aprobada aporta evidencia sobre esos casos; no demuestra automaticamente todos los comportamientos posibles ni inteligencia general del modelo que produjo una propuesta.

## Comprobacion
Elegir un fallo concreto y un caso que lo detecte antes de modificar el codigo. Comprobar que el mismo caso pasa tras la correccion y que las expectativas no se derivaron de la implementacion defectuosa.

## Fuente
Nota propia introductoria.
https://doc.rust-lang.org/book/ch11-01-writing-tests.html

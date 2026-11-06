# Escuela Rust v1: entrenamiento y agente acotado

Primer paso de Boddy propio hacia programación: un corpus Rust para Attention
y un ciclo de elección, propuesta, verificación y revisión. No usa Qwen.
No equivale a un agente capaz de modificar proyectos ni a dominar Rust.

## Qué se implementa

`boddy-rust-tasks` es no_std: define seis ejercicios i32 (suma, resta, producto,
mayor, menor y absoluto), nueve expresiones tipadas y cinco casos por ejercicio.
El verificador evalúa exclusivamente ese enum con aritmética comprobada. No
parsea Rust arbitrario, invoca rustc, ejecuta código generado ni consulta red.
Los valores esperados están escritos aparte del entrenamiento y no se envían
al modelo. Los casos incluyen ceros, negativos y operandos que distinguen las
soluciones; no prueban equivalencia para todos los valores i32.

`boddy-lab::attention::Ranker` puntúa cada expresión por pérdida media por byte
condicionada al prefijo de la tarea. Esto es ranking de opciones conocidas,
no generación libre de código. El límite de 16 bytes de contexto sigue vigente.

`boddy-agent::rust_school` ordena esos candidatos y usa Mission, Policy y Task:
leer catálogo -> plan -> proponer -> verificar. Si falla, revisa y prueba el
siguiente candidato; máximo tres intentos por ejercicio. Una propuesta solo
termina Completed cuando supera los cinco casos. Las tareas agotadas y la suite
incompleta quedan Stopped; no se aprueban por insistencia.

Guarda report.json y solutions.rs en `data/rust-school/school-<id>`, ignorado por
Git. El archivo Rust contiene únicamente plantillas de la biblioteca que pasaron
sus casos, con aviso de alcance; no se compila ni se aplica a otro proyecto.
El registro conserva ranking, intentos, resultados, hashes/procedencia del modelo,
política y rutas. No hay reanudación de tareas, entrenamiento automático a partir
del informe, ejecución general ni sandbox de procesos implementados en esta escuela.

## Enseñar y comparar

Perfil `rust-v1`: `training/corpora/boddy-rust-v1.txt`, con funciones pequeñas y
comentarios de objetivo. Validación de bytes en `training/evals/boddy-rust-v1.txt`,
con nombres diferentes y los mismos conceptos básicos. La suite del verificador
usa otros operandos. Son ejemplos y tareas conocidos durante el desarrollo:
constituyen un benchmark educativo, no una prueba ciega de programación general.

Attention conserva `general-v1` como perfil original. Si se continúa sin indicar
perfil se conserva el del padre; checkpoints antiguos se consideran general-v1.
Elegir explícitamente otro perfil conserva los pesos, reinicia el muestreador a
42 y registra `transfer_profile`, sin modificar el padre. Continuar en el mismo
perfil restaura el muestreador y exige el mismo hash de entrenamiento. Los pasos
acumulados incluyen toda la línea de versiones, incluso otros perfiles.

En la ventana, **Rust** permite cargar versiones, educar 300 pasos en GPU y
resolver ejercicios con uno a tres intentos. Se muestran aciertos de primera
elección, aciertos tras búsqueda, propuestas aceptadas y registro completo.
**Atencion** también permite seleccionar el perfil general o Rust.

```powershell
'{"action":"train","steps":300,"profile":"rust-v1"}' | .\target\debug\boddy-cli.exe atencion
'{"action":"tasks"}' | .\target\debug\boddy-cli.exe escuela-rust
# Usar un attn-<id> real:
'{"action":"run","id":"attn-<id>","max_attempts":3}' | .\target\debug\boddy-cli.exe escuela-rust
```

Medición RTX 3060 desde un Attention general de 300 pasos, seguido de 300 pasos Rust:

| Medida | Antes | Después |
|---|---|---|
| Pérdida en texto de entrenamiento Rust | 4,4870 | 3,8320 |
| Pérdida en texto de validación Rust | 4,8796 | 4,1192 |
| Primera elección correcta | 0/6 | 1/6 |
| Resueltas con hasta tres intentos | 2/6 | 2/6 |

Quedaron propuestas para suma y resta, verificadas solo en los casos definidos;
cuatro ejercicios pendientes. La suite terminó Stopped. La búsqueda aporta parte
del resultado; bajar pérdida no bastó para mejorar su cobertura. No se declara
que el modelo ya sepa programar Rust. Evidencia: `evals/rust-school-v1-rtx3060.json`.
Repetir con `tools/Test-RustSchool.ps1`; sin padre crea una base general de 100
pasos, por lo que esos números no tienen por qué coincidir con esta observación.

Pruebas: soluciones distinguibles, desbordamientos rechazados, revisiones acotadas,
rechazo de rankings inválidos, perfiles y ranking limitados. La prueba nativa de
ventana entrenó el perfil y resolvió la suite sin alterar el chat. Verificación
funcional; no es inspección visual de la interfaz.

Siguiente alcance: tareas nuevas, corpus más diverso y contexto suficiente para
las instrucciones. Para Rust arbitrario harán falta análisis de código, compilador
y ejecutor aislado con límites; esos componentes siguen pendientes.
Semántica contrastada con [operadores de Rust](https://doc.rust-lang.org/reference/expressions/operator-expr.html)
y [operaciones comprobadas i32](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_add).

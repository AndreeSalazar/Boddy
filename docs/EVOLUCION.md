# Evolución medible del agente

Fortalecer Boddy implica mejorar recuperación, contratos, comprobaciones y resultados
observables. El checkpoint Qwen y su cuantización permanecen intactos. Esta
sección describe el investigador de lectura; los laboratorios propios sí entrenan
Attention, y la escuela Rust añade un ciclo limitado de propuesta/verificación.
Ver ESCUELA-RUST.md. Sus registros tampoco se convierten automáticamente en datos correctos.

## Cambios

- Resuelve consultas de componentes: policy.rs puede buscar policy y reconocer
  entradas existentes en crates/boddy-policy. Solo añade nombres candidatos;
  después siguen los límites y comprobaciones de lectura.
- El host prepara anclas con texto y ubicación verdaderos. El modelo elige IDs y
  redacta conclusiones; no inventa texto de citas ni sus números de línea.
- Se excluyen cabeceras de función de las anclas y se detectan ciertas afirmaciones
  de permiso incompatibles con un inicializador false. Es una heurística lingüística
  acotada, no comprensión general ni garantía de verdad.
- Si el informe falla por JSON, ID o contrato inválidos, el modelo recibe una sola
  oportunidad de corregir con evidencia y feedback. El mismo verificador vuelve
  a comprobarlo. Si falla otra vez, Stopped; no se aprueba por insistencia.
- El registro guarda límites, respuestas originales, intentos y errores. El núcleo
  pasa por revisión y vuelve a verificar, manteniendo sus permisos anteriores.
- La ventana muestra correcciones usadas. Evidencia permite inspeccionar el registro.

Límites: cuatro llamadas al modelo, seis herramientas de lectura/búsqueda y tres
relecturas para comprobar hashes. Hasta tres archivos y 18 anclas. Se mantiene la
inferencia local existente, 4096 tokens de contexto y salida de 512 por llamada.
Una revisión puede aumentar tokens y latencia; no se declara ahorro de VRAM.

## Medición

evals/research-v1.json fija tres objetivos y sus fuentes esperadas: la plantilla de
Programar sobre budget.rs, presupuesto de tokens y permisos de policy.
La métrica comprueba Completed con citas verificadas y lectura de la fuente esperada.
No puntúa la corrección semántica de las conclusiones.

La línea base previa completó 1/3. Una corrección de texto libre no resultó fiable
en la repetición; motivó el protocolo de anclas. La primera ronda final completó
3/3, con 3, 4 y 3 llamadas respectivamente. La revisión humana encontró aun así una
conclusión falsa de permisos con cita auténtica; motivó los filtros adicionales.
Es un conjunto pequeño y conocido durante
el desarrollo, no evidencia de precisión general ni un examen independiente.
El modelo puede seguir produciendo conclusiones superficiales o incorrectas con
citas auténticas. semantic_review sigue pending.

La ronda final, tras los filtros, volvió a obtener 3/3 en esta métrica, con 3, 4 y 4
llamadas. En el caso de permisos se rechazó la afirmación positiva contra false;
el modelo la retiró en la revisión. Esto no prueba que el alcance de todas sus
otras conclusiones sea correcto. Se mantienen casos e intentos para revisión humana.

Los casos están versionados; las ejecuciones y objetivos se guardan fuera de Git
en data/evaluations y data/tasks. Para repetir con motor cargado y host compilado:

```powershell
.\tools\Evaluate-Research.ps1 -RepairAttempts 1
.\tools\Evaluate-Research.ps1 -RepairAttempts 0
```

Desactivar la revisión mide su coste dentro de la misma implementación; no reconstruye
la versión antigua: búsqueda y anclas siguen mejoradas. Los resultados incluyen
estado, fuente esperada, llamadas, tokens y registro. Un fallo queda como resultado.

## Siguiente criterio de evolución

Evaluar contenido con respuestas de referencia y tareas nuevas, además de citas.
Después mejorar selección de fragmentos y cobertura del comportamiento. Entrenar
adaptadores requerirá datos revisados, un experimento reproducible y evaluación
separada; los registros actuales no son ejemplos correctos automáticamente.
La ejecución de cambios todavía necesita el ejecutor aislado.

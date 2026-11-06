# Frontera del adaptador WGPU — pendiente

Este nodo reserva exclusivamente la implementación del backend de cómputo GPU.
Todavía no contiene runtime, shaders ni inferencia. No se añade una dependencia
pesada a todas las crates para aparentar una capacidad que aún no existe.

Entrada futura: operaciones tensoriales, buffers identificados, límites de memoria
y cancelación. Salida: resultados y medidas de recursos. Device, Queue, Pipeline
y tipos concretos de WGPU quedan dentro del adaptador; ninguna API del núcleo los
expone. El host será dueño de su ciclo de vida.

Antes de conectarlo: seleccionar formato y modelo compatibles, comparar operaciones
con referencia CPU, probar pérdida de dispositivo, límites y consumo pico en RTX 3060.
El aislamiento de código no confiable pertenece al ejecutor del SO, separado de GPU.

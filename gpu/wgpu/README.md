# Adaptador WGPU: entrenamiento educativo

`boddy-wgpu` implementa forward, gradientes y SGD en tres pasadas WGSL separadas.
Usa WGPU 30.0.1 con Vulkan, sin superficie. Device, Queue, Buffer y Pipeline
permanecen privados en esta crate. Recibe configuración, pesos y lotes; devuelve
pesos, pérdida y gradientes. Rechaza adaptadores CPU como prueba de GPU.

`boddy-neural` es la referencia CPU no_std. `boddy-lab` exige paridad de pérdida,
todos los gradientes y una actualización antes de entrenar. Estos entrenadores
no ejecutan Qwen. La inferencia del chat sigue usando llama.cpp.

El módulo `transformer` añade un intérprete escalar de la cinta no_std de
`boddy-transformer`: forward, backward y SGD con recorte de norma. Opera en
una sola invocación, sin atomics ni carreras; es referencia funcional, sin
afirmaciones de rendimiento frente a CPU. Ver docs/ATTENTION.md.

Arquitectura, lote, tasa e índices están acotados; lectura GPU con espera máxima
de 30 segundos. Recuperación de dispositivo, cancelación cooperativa, entrenamiento
grande y medición de pico VRAM siguen pendientes. Buffers declarados no incluyen
todos los recursos del driver. WGPU no es un sandbox del SO.

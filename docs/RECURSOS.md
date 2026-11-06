# Medir para optimizar Boddy

La ventana muestra medidas reales reportadas por el motor y una muestra de GPU.
Los controles cambian el presupuesto de la petición; no reconfiguran el servidor
ni entrenan el modelo. Contexto por defecto: 2048 tokens; salida máxima: 256.

| Medida | Significado | Qué no demuestra |
|---|---|---|
| Entrada | Tokens del prompt completo, incluido sistema, historial y mapa opcional | No son solo los caracteres recién escritos |
| Caché | Parte de la entrada reutilizada por el motor | No se suma otra vez a los tokens de entrada |
| Salida | Tokens generados en la respuesta | No representa todo el trabajo del motor |
| Acumulado | Suma de las peticiones exitosas de este chat | No es contexto simultáneamente residente; repetir historial vuelve a contarlo |
| Recorte | Mensajes antiguos y tokens excluidos del prompt de esta petición | No es compresión sin pérdida ni aprendizaje |
| tok/s | Velocidad de generación informada por el motor | No es velocidad integral del sistema ni benchmark de calidad |
| Segundos | Tiempo del adaptador, incluyendo conteo y generación | Excluye consulta al grafo previa y telemetría posterior |
| VRAM | Dispositivo NVIDIA 0 completo al terminar | Incluye escritorio y otras apps; no es consumo del proceso ni pico |

Los contadores principales vienen de `usage`; el conteo previo aplica la plantilla
de chat y el tokenizer del servidor. El host elimina pares antiguos completos hasta
que entrada + salida reservada + margen de 64 tokens quepan en el presupuesto.
La petición actual nunca se corta silenciosamente: si no cabe, se rechaza con un
mensaje para ampliarlo o acortarla. Los textos anteriores siguen visibles en pantalla,
pero los turnos recortados ya no se envían. El host devuelve el historial retenido
para que la ventana y el modelo coincidan en los siguientes turnos.

Hay límites adicionales de transporte: 24 mensajes, 8000 caracteres por mensaje,
16000 en total y 64 KiB de JSON de entrada. Son defensas de memoria; no sustituyen
el conteo de tokens. Una respuesta con `finish_reason=length` se marca como cortada.
Los acumulados solo suman respuestas exitosas; una cancelación o fallo puede haber
consumido cómputo en el motor que no llegue a contabilizarse en la ventana.

## SUPER GRAFO como selector

La casilla **Mapa de Boddy** consulta `resumen` sobre este proyecto y aporta
únicamente archivos indexados, dependencias y ciclos. No carga el repositorio
completo ni autoriza ejecución. Los números se incluyen en la entrada y cuentan
como tokens. No equivalen a entender archivos o probar el programa.

Siguiente ampliación: consultas por archivo y recuperación de fragmentos pertinentes,
con rutas y versiones. Requiere una evaluación de recuperación antes de afirmar que
se mantiene la calidad usando menos contexto. El mapa no sustituye los pesos neuronales.

## Qué se puede cargar bajo demanda

La analogía con un motor gráfico es útil para documentos, herramientas y contexto:
seleccionar primero, traer solo lo pertinente y reutilizar resultados. El modelo
denso Qwen actual sigue necesitando sus cálculos de pesos; no se pueden omitir
neuronas arbitrarias sin cambiar su comportamiento.

Cuantización, caché KV, reparto CPU/GPU y arquitecturas con expertos son técnicas
distintas. Ahorrar VRAM mediante descarga a RAM puede aumentar la latencia por
transferencias. Con 16 GB de RAM también hay un presupuesto del sistema anfitrión.
El presupuesto de contexto de la ventana NO cambia la reserva KV del servidor,
que sigue configurado para 8192 tokens. No se afirma que recortar entrada libere
inmediatamente esa VRAM. La integración WGPU continúa pendiente.

Fuentes: [API del servidor llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server)
para plantilla, tokenización, caché y métricas;
[estrategias de caché](https://huggingface.co/docs/transformers/main/en/kv_cache)
para las diferencias entre memoria, reutilización y descarga. Las decisiones de
presupuesto y los límites de esta implementación son propios de Boddy.

## Evidencia de esta entrega

- Prompt con mapa: preconteo de 170 tokens; el motor informó 170 de entrada,
  8 de salida y 88 reutilizados de caché.
- Historial de prueba: 1737 tokens antes, 382 después; se retiraron 10 mensajes
  y 1355 tokens de entrada. La petición actual se conservó y respondió FaroAzul.
- Ventana: entrada 109, salida 4 y caché 94; mostró una muestra de GPU disponible.

Son casos funcionales del equipo disponible, no una evaluación general de calidad,
un benchmark repetido ni evidencia de entrenamiento de los pesos.

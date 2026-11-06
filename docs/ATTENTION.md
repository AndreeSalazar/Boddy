# Boddy Attention 0.1: primer decoder propio

Experimento independiente de Tiny 1.0 y del chat Qwen. Sus 10.544 parámetros
empiezan al azar; no importa pesos ni respuestas de Qwen. La nueva arquitectura
todavía no demuestra lenguaje útil: la primera muestra greedy repite espacios.

## Arquitectura y fronteras

UTF-8 -> bytes (256 tokens) -> embeddings 16D + posiciones aprendidas -> RMSNorm
-> atención causal Q/K/V de una cabeza -> proyección O + residual -> RMSNorm
-> FFN 16/32/16 tanh + residual -> RMSNorm -> 256 logits y entropía cruzada.

El contexto es **16 bytes**, no caracteres. Se calcula la última posición para
predecir el siguiente byte; la cinta solo incluye posiciones presentes/pasadas.
No hay RoPE, SwiGLU, varias cabezas/capas, dropout ni entrenamiento distribuido.

`boddy-transformer` contiene matemáticas y autodiferenciación no_std con alloc y
libm. `gpu/wgpu` interpreta su cinta con forward, backward, recorte de norma a 1
y SGD de tasa 0,03, lote de un contexto, hasta 1.000 pasos por petición. Antes de
entrenar se comparan pérdida, todos los gradientes y una actualización con CPU.
Umbrales absolutos: 1e-4, 1e-3 y 1e-4 respectivamente.

El intérprete GPU usa una sola invocación secuencial para evitar carreras. Es una
referencia funcional, sin afirmación de ventaja de velocidad sobre CPU. Kernels
tensoriales paralelos y benchmarks son trabajo futuro. La generación usa CPU.

## Datos, texto y archivos

El perfil original es `general-v1`. La escuela añade `rust-v1`, con corpus y
validación de texto separados (ESCUELA-RUST.md). El selector permite continuar
en el perfil del padre o elegir otro: cambiarlo transfiere pesos y reinicia el
muestreador, registrando `transfer_profile`. El JSON guarda perfil y hash de
validación; los tamaños iniciales de la tabla de esta página son históricos,
anteriores a esos campos. Los pesos siguen ocupando 42.176 bytes.

El host también expone un Ranker de continuaciones conocidas por pérdida media
por byte. No recibe casos ni respuestas del verificador de ejercicios. Su uso
en búsqueda de expresiones no equivale a generación libre de programas.

Corpus educativo propio `training/corpora/boddy-attention-v1.txt`, con español,
acentos y algunos símbolos. Se separa el último 20 % de líneas antes de crear
ventanas; se evalúan los mismos 64 contextos de cada partición antes y después.
Es validación pequeña/relacionada, no mide razonamiento ni programación. Las
pérdidas no se comparan directamente con Tiny: corpus y tokenizer son distintos.

La codificación representa cualquier cadena Unicode válida sin pérdida usando
bytes UTF-8; eso no implica conocer todos los idiomas. La generación greedy
admite prefijo de hasta 512 bytes, salida de hasta 128 bytes y una máscara de
validez UTF-8. Excluye controles salvo tabulación y salto de línea; si queda un
carácter incompleto, descarta esa cola e informa `discarded_utf8_tail_bytes`.
Conserva el prefijo completo en la salida; solo sus últimos 16 bytes entran al
modelo en cada predicción. La máscara no corrige el significado del texto.

`data/attention/attn-<id>` contiene checkpoint.json, weights.f32le y metrics.json.
Formato separado de Tiny v3: pesos no intercambiables ni deduplicados todavía.
Guarda configuración, semilla, muestreador, pasos, padre y hashes. Se comprueban
arquitectura, tokenizer, tamaño, SHA256 y valores finitos. La recarga exige pesos
idénticos bit a bit y la misma muestra. Cada versión conserva el padre intacto.

## Usarlo y comprobarlo

En la ventana, elegir **Atencion**: crear, seleccionar y continuar un modelo,
consultar métricas o generar desde un prefijo. Conversar conserva Qwen/Tiny;
Attention se presenta como experimento, sin herramientas ni historial.

```powershell
cargo build -p boddy-cli --offline
'{"action":"train","steps":300}' | .\target\debug\boddy-cli.exe atencion
'{"action":"list"}' | .\target\debug\boddy-cli.exe atencion
# Sustituir attn-<id> por una version real:
'{"action":"train","steps":100,"parent_id":"attn-<id>"}' | .\target\debug\boddy-cli.exe atencion
'{"action":"generate","id":"attn-<id>","prefix":"Boddy: ","max_tokens":64}' | .\target\debug\boddy-cli.exe atencion
```

Las tuberías PowerShell con Unicode deben enviar/leer UTF-8. La ventana y
`tools/Test-Attention.ps1` lo configuran explícitamente. El valor ASCII por defecto
de algunas versiones de PowerShell pierde acentos al enviar a programas nativos.

Resultados observados en RTX 3060:

| Medida | Resultado |
|---|---|
| Pérdida de entrenamiento, 300 pasos | 5,5408 -> 3,9268 |
| Pérdida de validación, 300 pasos | 5,5679 -> 4,1217 |
| Error máximo de gradientes GPU/CPU inicial | 4,77e-7 |
| Error máximo SGD | 7,45e-9 |
| Cinta | 43.321 nodos |
| Buffers GPU declarados | 1.081.976 bytes |
| JSON inicial / pesos f32 | 508 / 42.176 bytes |
| 200 pasos frente a 100 + 100 | Mismos pesos y muestreador; padre intacto |

Los buffers excluyen driver, compilador, staging y otros procesos: no son RAM o
VRAM total ni pico. Evidencia: `evals/attention-v1-rtx3060.json` y
`evals/attention-continuation-v1-rtx3060.json`. Reducir pérdida no demuestra chat.

Pruebas CPU: diferencias finitas de gradientes, causalidad, descenso de pérdida,
roundtrip Unicode y máscara UTF-8. El test nativo de ventana entrenó 300 pasos,
continuó a 400 y generó con prefijo Unicode sin alterar el historial. Verificación
funcional, sin inspección visual del diseño. Recuperación de dispositivo,
cancelación cooperativa y kernels paralelos siguen pendientes. WGPU no es sandbox.

Inspiración matemática: [Attention Is All You Need](https://arxiv.org/abs/1706.03762)
y [RMSNorm](https://arxiv.org/abs/1910.07467). No reproduce la arquitectura ni el
entrenamiento completo de Qwen.

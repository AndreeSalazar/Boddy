# Boddy 1.0 — chat local

La ventana ofrece Conversar, Programar, Mi PC, Evidencia, Laboratorio, Atencion y Rust. Programar muestra
fuentes y fragmentos y permite investigar la selección. Mi PC abre herramientas
registradas por acción manual; Evidencia recupera las tareas guardadas.
[Controles y alcance](../docs/ESCRITORIO.md).

Doble clic en `Boddy.vbs`, o desde la raíz del proyecto:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -STA -File .\window\Start-Boddy.ps1
```

El lanzador compila el host, inicia el modelo local y abre el chat. Se necesitan
Rust, el lanzador compilado (`cargo build -p boddy-window --offline`) y el modelo preparado en este proyecto. El bypass solo afecta este proceso
PowerShell; no modifica la política permanente de Windows.

Escribe normalmente. Enter envía; Shift+Enter agrega una línea. Puedes cancelar
la espera y empezar un nuevo chat. El historial reciente se conserva en memoria
con un presupuesto acotado y se recorta por turnos; no se guarda en disco.

Puedes elegir presupuesto de contexto y salida máxima. Se muestran tokens de
entrada/salida, caché, acumulados, recortes, velocidad y una muestra de VRAM.
La casilla Mapa de Boddy añade solo cifras seleccionadas de SUPER GRAFO.
El historial enviado se recorta por tokens reales en el host y se sincroniza con
la ventana. Los turnos antiguos siguen visibles pero pueden salir del contexto.
Estos controles no modifican la reserva GPU del servidor. [Medidas y límites](../docs/RECURSOS.md).

Activa **Investigar proyecto (lectura)** para dar un objetivo sobre Boddy.
El modelo propone búsquedas y fuentes; el host lee fragmentos y comprueba citas.
Los controles de presupuesto del chat se desactivan en ese modo porque usa cupos
propios. El resultado muestra estado y ruta del registro local. Las conclusiones
requieren revisión aunque las citas hayan pasado. [Alcance](../docs/AGENTE-LECTURA.md).

El investigador puede corregir una vez un informe rechazado. La ventana muestra
las correcciones usadas y el registro conserva los dos intentos y sus errores.
La comprobación de citas se mantiene igual de estricta después de corregir.

Boddy 1.0 es el nombre del asistente. El modelo base es Qwen2.5-Coder-7B-Instruct
Q4_K_M, ejecutado por llama.cpp b12110 sobre Vulkan. El motor solo escucha en
127.0.0.1:17861. No requiere cuenta ni clave de un proveedor. El chat envía texto
al motor local. El modo investigación ofrece lecturas acotadas del proyecto a
través del host; no dispone de ejecución de código.

Esta primera interfaz muestra la respuesta completa al terminar. Cancelar cierra
el cliente; la liberación inmediata del trabajo del servidor es de mejor esfuerzo.
El modelo se mantiene cargado para reutilizarlo; puedes apagarlo con
`tools/Stop-Model.ps1` cuando termines.

Primera preparación en otra instalación:

```powershell
.\tools\Install-Model.ps1
```

Descarga unos 4,7 GB de pesos más el motor y verifica SHA256 según
`models.lock.json`. Se conservan fuera de Git. Los pesos no se entrenaron en este
proyecto. El laboratorio independiente entrena Boddy Tiny desde cero mediante
WGPU: elegir pasos y pulsar Entrenar en GPU. Muestra pérdida, tokens procesados,
buffers, GPU y muestra de texto; guarda pesos en data/lab. Elegir una version
guardada permite continuar y comparar sin sobrescribir la anterior. Conversar
incluye selector Qwen/Tiny: Tiny solo continua caracteres ASCII de manera
experimental, sin historial ni herramientas. Actualizar recarga las versiones;
cambiar modelo reinicia historial y contadores. Puede abrirse directamente con
Boddy.ps1 sin iniciar el servidor Qwen.
Los nuevos checkpoints usan JSON pequeño y pesos binarios compartidos cuando
son idénticos. Laboratorio muestra tamaños de disco; los modelos antiguos siguen
cargando. Para copiar una versión nueva hay que incluir también su blob de pesos.
[Arquitectura, comprobaciones y límites](../docs/MODELO-PROPIO.md).

**Atencion** contiene el experimento Boddy Attention 0.1, separado del chat y
Tiny. Usa bytes UTF-8, embeddings y una cabeza causal. Permite crear pesos desde
cero, continuar una versión guardada, consultar métricas y generar desde un
prefijo Unicode. Entrena por WGPU y genera por CPU. Aún no demuestra lenguaje
útil ni programación. [Detalles y evidencia](../docs/ATTENTION.md).

**Rust** permite educar un Attention con el perfil Rust y ejecutar seis ejercicios
de elección entre expresiones conocidas. Muestra primera elección, resultados
tras búsqueda, propuestas aceptadas y registro. Los fallos quedan pendientes.
Atencion incluye selector de corpus general/Rust; cambiar perfil conserva al
padre y crea otra versión. [Alcance exacto](../docs/ESCUELA-RUST.md).

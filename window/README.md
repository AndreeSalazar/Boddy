# Boddy 1.0 — chat local

La ventana ofrece Conversar, Programar, Mi PC y Evidencia. Programar muestra
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
proyecto. WGPU sigue siendo una integración futura independiente; Vulkan no es WGPU.

# Boddy 1.0 — chat local

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

Boddy 1.0 es el nombre del asistente. El modelo base es Qwen2.5-Coder-7B-Instruct
Q4_K_M, ejecutado por llama.cpp b12110 sobre Vulkan. El motor solo escucha en
127.0.0.1:17861. No requiere cuenta ni clave de un proveedor. El chat envía texto
al motor local; no dispone de herramientas para leer archivos o ejecutar código.

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

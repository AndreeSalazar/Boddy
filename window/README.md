# Cuadro de comandos Windows

Interfaz nativa WinForms independiente del motor. Selecciona un comando y su
parámetro, muestra resultados y mantiene la ventana disponible mientras trabaja.
Solo llama a la CLI, que reutiliza `boddy-app`; no ejecuta una shell arbitraria.

```powershell
cargo build -p boddy-cli --offline
powershell.exe -NoProfile -ExecutionPolicy Bypass -STA -File .\window\Boddy.ps1
```

`fuentes` admite una búsqueda; `grafo` admite resumen/fallos/ciclos. `demo` y
`proposito` no reciben parámetros. Esta UI usa controles Windows; la integración
WGPU para inferencia permanece separada en `gpu/wgpu` y pendiente de implementar.
La UI usa el ejecutable debug del proyecto; vuelve a compilar tras cambiar Rust.

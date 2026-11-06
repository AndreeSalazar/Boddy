# Reglas de construcción

- Mantener una sola responsabilidad por crate y dependencias acíclicas explícitas.
- Núcleo, misión, permisos y contratos siguen siendo `no_std`; no incorporan UI,
  controladores, disco, red, intérpretes ni dependencias del host.
- `window` contiene la interfaz Windows; consume comandos de aplicación, nunca
  duplica reglas de tareas o implementa inferencia.
- WGPU pertenece exclusivamente a `gpu/wgpu`. Los demás módulos consumen contratos;
  no importar tipos de WGPU en el núcleo ni presentar GPU como sandbox.
- Cada componente debe figurar en `super-grafo/nodos.json`, con propósito,
  dependencias y estado real. Actualizar mapa y notas al cambiar fronteras.
- Componentes futuros se documentan como pendientes; no declarar capacidades sin
  implementación y evidencia. Optimizar con mediciones de comportamiento y recursos.
- Comprobar contratos con tests, Clippy y compilación portable antes de guardar cambios.
- Separar commits de fundamentos y cambios estructurales. No ejecutar código de
  proyectos o abrir red por el solo hecho de que un modelo lo proponga.

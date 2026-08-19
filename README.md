# marvinvirus

Limpiador de Windows (Tauri v2 + Vue 3). Público objetivo: usuarios no técnicos.

## Módulos v1

- **Temporales y caché del sistema** — `%TEMP%`, `C:\Windows\Temp`, Windows Update download cache, miniaturas, WER, Prefetch (opcional).
- **Cachés de navegadores** — Chrome / Edge / Firefox. Solo caché de disco. Nunca cookies, contraseñas ni historial.
- **Desinstalación de programas** — Registro `HKLM/HKCU\...\Uninstall` (+ `WOW6432Node`). Lanza el desinstalador nativo del programa.
- **Duplicados y archivos grandes** — Documentos, Descargas, Escritorio, Imágenes, Vídeos. Agrupación por tamaño → hash BLAKE3 sobre candidatos.
- **Gestión de arranque** — `Run`/`RunOnce` (HKLM+HKCU), carpeta Inicio, apps de inicio de Windows, tareas programadas. Deshabilita, no elimina.

## Arquitectura

- `src-tauri/` — proceso principal Tauri. Sin privilegios. UI, escaneos, borrado en espacio de usuario.
- `helper/` — binario Rust independiente. Elevado bajo demanda vía `ShellExecuteW` con verbo `runas`. Ejecuta una lista cerrada de operaciones y termina.
- Papelera de reciclaje siempre (nunca borrado permanente en v1). Crate `trash` bajo el capó (usa `IFileOperation`).

## Compilar

Windows y macOS producen artefactos en GitHub Actions (matriz). Local:

```
npm ci
npm run build
cd src-tauri && cargo tauri build
```

macOS compila el mismo código con módulos de escaneo/limpieza stub (devuelven listas vacías). Sirve para desarrollo de UI y CI, no para uso real.

## Estado

v1 en construcción. Fases 1–5 según `Audit de proyecto — Limpiador Windows`.

# 🚀 AGENTS.MD - Gate Runner (Rust + Macroquad)

## 📌 Visión del Proyecto
Desarrollar un videojuego educativo en **Rust** usando **Macroquad** que combine:
1. **Mecánica de Multiplicación / Corredor:** Movimiento horizontal para atravesar puertas flotantes con modificadores (`+N`, `-N`, `xN`, `/N`).
2. **Shoot 'em Up:** Disparo automático hacia arriba contra hordas o bloques enemigos que descienden.
3. **Tower Defense / Línea de Defensa:** Prevenir que los enemigos crucen la parte inferior de la pantalla o destruyan tu base.

---

## 🛠️ Stack Tecnológico & Dependencias
- **Lenguaje:** Rust (Edición 2021)
- **Motor / Gráficos:** `macroquad` (0.4+) con la feature `audio`
- **Web:** compilación a WebAssembly (`wasm32-unknown-unknown`), PWA instalable con service worker, despliegue a GitHub Pages con GitHub Actions.
- **Enfoque de diseño:** Código limpio, iterativo, bien comentado y sin librerías externas complejas (gráficos con primitivas geométricas 2D y audio sintetizado con matemáticas; sin assets de imagen ni de sonido).

---

## 🎯 Principios de Código & Guía de Estilo
- **Simplicidad ante todo:** Priorizar claridad educativa sobre micro-optimizaciones prematuras.
- **Comentarios explicativos:** Explicar el *porqué* de cada estructura de Rust (`Vec`, `struct`, `match`, ownership en bucles).
- **Modularidad progresiva:** Iniciar en un solo archivo `main.rs` para captar la idea global, y refactorizar a módulos (`entities.rs`, `game.rs`, `audio.rs`...) conforme crezca el proyecto.
- **Delta Time (`get_frame_time()`):** Todo el movimiento debe estar basado en tiempo delta para asegurar la misma velocidad en cualquier framerate.
- **Compatible con la web:** Evitar APIs que no existan en WASM (`std::thread`, `std::time::Instant`, sockets...). `std::fs` solo se usa en `highscore.rs` y falla con elegancia en el navegador.

---

## 🗺️ Hoja de Ruta / Fases de Desarrollo

### 🟢 Fase 1: Fundamentos y Ventana Principal ✅
- [x] Configuración inicial de Cargo y `Cargo.toml`.
- [x] Inicialización de la ventana de Macroquad (`Conf`), 480x800 vertical.
- [x] Bucle principal (`loop`) y fondo de pantalla.

### 🟡 Fase 2: El Jugador y Movimiento ✅
- [x] Representación visual del jugador/torreta.
- [x] Control horizontal (Teclas Izquierda/Derecha o A/D).
- [x] Definición de límites de la pantalla (pantalla como carril).

### 🔵 Fase 3: Sistema de Puertas (Gates) ✅
- [x] Estructura `Gate` con posición, tamaño, tipo de operación (`Add`, `Subtract`, `Multiply`) y valor.
- [x] Generación y descenso continuo de puertas.
- [x] Detección de colisión (AABB: *Axis-Aligned Bounding Box*) entre el jugador y las puertas.
- [x] Aplicación del modificador al contador/ejército del jugador.

### 🔴 Fase 4: Disparo y Sistema de Enemigos (Shmup + Tower Defense) ✅
- [x] Disparo del jugador (proyectiles que viajan hacia arriba, más balas con más soldados).
- [x] Generación de enemigos que bajan con puntos de vida (HP).
- [x] Colisiones Proyectil vs Enemigo.
- [x] Línea de defensa inferior: Pérdida de vidas / Game Over si un enemigo cruza la línea.

### 🟣 Fase 5: UI, Pulido y Game Loop Completo ✅
- [x] Marcadores en pantalla: Soldados, vidas, nivel, puntos y récord.
- [x] Pantallas de estado: `Menu`, `Playing`, `Paused`, `GameOver`.
- [x] Efectos visuales (colores por tipo de puerta, partículas, estrellas con parallax).
- [x] Power-ups (disparo rápido, escudo, vida extra).
- [x] Sonido y música sintetizados (sin archivos de audio).
- [x] Récord persistente en `highscore.txt` (solo escritorio).

### 🌐 Fase 6: Web, PWA y Despliegue ✅
- [x] Compilación a WebAssembly (`wasm32-unknown-unknown`) con flag `--import-undefined` en `.cargo/config.toml` (requerido por el audio en la web).
- [x] Carpeta `web/`: `index.html`, runtime `mq_js_bundle.js` vendorizado, íconos.
- [x] PWA: `manifest.webmanifest` + botón de instalación (`beforeinstallprompt`).
- [x] Juego sin conexión: service worker `sw.js` con estrategia *cache first*.
- [x] CI/CD: `.github/workflows/deploy.yml` compila y publica en GitHub Pages en cada push a `main`.

### 💡 Fase 7: Ideas Futuras
- [ ] Enemigo jefe cada 5 niveles, con barra de vida.
- [ ] Distintos tipos de enemigos (rápidos, blindados, zigzag).
- [ ] Controles táctiles / gamepad (importante para la PWA en móviles).
- [ ] Récord persistente en la web (`localStorage`).
- [ ] Pantalla de tabla de récords.

---

## 📂 Estructura del Proyecto

```text
gaterunner/
├── Cargo.toml          # Dependencias (macroquad con feature "audio")
├── AGENTS.md
├── README.md
├── .cargo/
│   └── config.toml     # Flag de linker para el build de WebAssembly
├── .github/
│   └── workflows/
│       └── deploy.yml  # CI: compila a WASM y despliega a GitHub Pages
├── web/
│   ├── index.html          # Página del juego + botón "Instalar" (PWA)
│   ├── manifest.webmanifest# Nombre, colores e íconos de la PWA
│   ├── sw.js               # Service worker: cache para jugar offline
│   ├── mq_js_bundle.js     # Runtime de macroquad para el navegador
│   └── icon-*.png          # Íconos de la app (192, 512 y maskable)
└── src/
    ├── main.rs         # Ventana, loop principal y módulos
    ├── constants.rs    # Números de ajuste del juego
    ├── entities.rs     # Balas, enemigos, puertas, power-ups, estrellas
    ├── game.rs         # struct Game y lógica de la partida
    ├── game/
    │   └── render.rs   # Dibujo (menú, HUD, pausa, game over)
    ├── highscore.rs    # Leer / guardar el récord en un archivo
    ├── synth.rs        # Sintetizador de ondas y creación del WAV
    ├── audio.rs        # Efectos de sonido (Sfx) y reproductor (Sounds)
    └── music.rs        # Melodía de fondo
```

---

## 🌐 Builds y Despliegue

| Objetivo | Comando |
|----------|---------|
| Escritorio (debug) | `cargo run` |
| Escritorio (release) | `cargo run --release` |
| Web (WASM) | `cargo build --release --target wasm32-unknown-unknown` |

- El `.wasm` queda en `target/wasm32-unknown-unknown/release/gaterunner.wasm`; para probarlo se sirve junto con los archivos de `web/` desde un servidor estático (`http://localhost` o `https`, requisito del service worker).
- El despliegue a GitHub Pages es automático en cada push a `main` (workflow `deploy.yml`). En el repo debe estar configurado **Settings → Pages → Source = "GitHub Actions"**.
- El workflow reemplaza `__CACHE_VERSION__` en `sw.js` por el hash del commit, para que los jugadores reciban la versión nueva automáticamente.
- En la web, `std::fs` no funciona: el récord no persiste entre sesiones (falla silenciosamente, por diseño de `highscore.rs`).

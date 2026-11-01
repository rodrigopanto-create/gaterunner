# Gate Runner

Un juego 2D hecho en **Rust** con **[macroquad](https://macroquad.rs/)**, pensado como **proyecto educativo**: cada paso agrega una idea nueva y el código está comentado en español.

Combina la mecánica de las puertas de multiplicación (`x2`, `+5`, `-10`) con un *shoot 'em up* y un toque de *tower defense*: dispara a las oleadas de enemigos que bajan y evita que crucen tu línea.

## 🎮 Jugar en el navegador

El juego está publicado en GitHub Pages:

**➡ https://rodrigopanto-create.github.io/gaterunner/**

Además es una **PWA** (*Progressive Web App*):

- **Instalar:** al abrir la página aparece un botón **⬇ Instalar** (arriba a la derecha). Al pulsarlo, el juego se instala como una app con su propio ícono y ventana.
- **Jugar sin conexión:** un *service worker* guarda todos los archivos del juego en el navegador; después de la primera visita puedes jugar aunque no tengas internet.

> Nota: en la versión web el récord no se guarda entre sesiones (la versión de escritorio lo guarda en `highscore.txt`).

## Cómo se juega

- Tu torreta avanza sola; tú la mueves a izquierda y derecha.
- Cruza una **puerta** para cambiar tu número de soldados: `x2`, `x3`, `+N` te ayudan; `-N` te perjudica.
- Más soldados = más balas por disparo (hasta 9 a la vez).
- Destruye enemigos para ganar puntos y subir de nivel. Cada nivel los hace más resistentes, más rápidos y más frecuentes.
- Si un enemigo cruza la línea inferior, pierdes una vida. Con 0 vidas, game over.
- Algunos enemigos sueltan **power-ups** al morir:

| Ítem | Color | Efecto |
|------|-------|--------|
| **R** | Naranja | Disparo rápido durante 6 segundos |
| **S** | Azul | Escudo: bloquea al siguiente enemigo que se escape |
| **+** | Verde | Una vida extra (máximo 9) |

Tu mejor puntaje se guarda en `highscore.txt`.

## Controles

| Tecla | Acción |
|-------|--------|
| `←` `→` o `A` `D` | Mover la torreta |
| `ESPACIO` | Disparar |
| `ENTER` | Empezar (en el menú) |
| `P` o `ESC` | Pausar / continuar |
| `R` | Jugar otra vez (en game over) |
| `M` | Volver al menú (en game over) |
| `N` | Silenciar / activar el sonido |

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install) (edición 2021 o superior)
- **Linux:** la librería de audio ALSA.
  - Debian / Ubuntu: `sudo apt install libasound2-dev`
- Windows y macOS: no necesitan nada extra.

## Ejecutar

### Escritorio (nativo)

```bash
git clone https://github.com/rodrigopanto-create/gaterunner.git
cd gaterunner
cargo run --release
```

### Web (WebAssembly)

El juego se compila a WebAssembly con el target `wasm32-unknown-unknown`:

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

Para probarlo en local, junta el `.wasm` con los archivos de `web/` y sírvelos con cualquier servidor estático (el service worker requiere `http://localhost` o `https`):

```bash
mkdir dist
cp web/* dist/
cp target/wasm32-unknown-unknown/release/gaterunner.wasm dist/
# por ejemplo, con Python:
python -m http.server 8080 -d dist
```

> El archivo [.cargo/config.toml](.cargo/config.toml) añade el flag de linker `--import-undefined` que necesita el audio de macroquad en la web. No afecta a los builds nativos.

### Despliegue automático (GitHub Actions)

Cada *push* a `main` ejecuta el workflow [.github/workflows/deploy.yml](.github/workflows/deploy.yml), que compila el juego a WASM y publica la carpeta resultante en GitHub Pages.

Configuración necesaria (una sola vez): en el repositorio, **Settings → Pages → Source = "GitHub Actions"**.

## Estructura del proyecto

```
src/
├── main.rs          Ventana y loop principal
├── constants.rs     Números de ajuste (velocidades, vidas, probabilidades...)
├── entities.rs      Balas, enemigos, puertas, power-ups, estrellas
├── game.rs          struct Game y toda la lógica de la partida
├── game/
│   └── render.rs    Todo el dibujo (menú, HUD, pausa, game over)
├── highscore.rs     Leer y guardar el récord en un archivo
├── synth.rs         Sintetizador de ondas y creación del WAV
├── audio.rs         Efectos de sonido (Sfx) y reproductor (Sounds)
└── music.rs         Melodía de fondo
web/
├── index.html       Página del juego + botón de instalación de la PWA
├── manifest.webmanifest  Nombre, colores e íconos de la PWA
├── sw.js            Service worker: cache para jugar sin conexión
├── mq_js_bundle.js  Runtime de macroquad para el navegador (vendorizado)
└── icon-*.png       Íconos de la app
.cargo/
└── config.toml      Flag de linker para el build de WebAssembly
.github/workflows/
└── deploy.yml       CI: compila a WASM y despliega a GitHub Pages
```

## Todo se genera con código

El juego **no usa imágenes ni archivos de audio**:

- Los gráficos son formas dibujadas con macroquad (rectángulos y círculos).
- Los efectos de sonido y la música se **sintetizan con matemáticas** (ondas seno, cuadrada y ruido), se empaquetan como un WAV en memoria y se cargan con `macroquad::audio`.

## Ruta de aprendizaje

El proyecto se construyó paso a paso. Cada paso introduce conceptos de Rust y de desarrollo de juegos:

| Paso | Qué se agrega | Conceptos |
|------|---------------|-----------|
| 1 | Ventana y torreta que se mueve | Game loop, `delta time`, `clamp` |
| 2 | Disparar balas | `struct`, `Vec`, `retain` |
| 3 | Enemigos, colisiones, puntos y vidas | Números aleatorios, `Rect`, `filter` |
| 4 | Puertas `x2`, `+5`, `-10` y game over | `enum` con datos, `match`, `impl` |
| 5 | Dificultad progresiva | `struct Game`, métodos, niveles |
| 6 | Pulido visual | Partículas, trigonometría, transparencias |
| 7 | Menú, pausa y récord guardado | `enum State`, archivos, `Option` |
| 8 | Power-ups y fondo de estrellas | `retain` con efectos, parallax |
| 9 | Efectos de sonido | Síntesis de audio, formato WAV, eventos |
| 10 | Música de fondo en bucle | Notas MIDI, mezcla de ondas, volumen por estado |
| 11 | Código dividido en módulos | `mod`, `use`, `pub`, módulos hijos |
| 12 | Versión web instalable | WebAssembly, PWA, service worker, GitHub Actions |

## Ideas para seguir

- Enemigo jefe cada 5 niveles, con barra de vida
- Distintos tipos de enemigos (rápidos, blindados, zigzag)
- Música que cambie con el nivel
- Pantalla de tabla de récords
- Controles táctiles o de gamepad
- Guardar el récord en la web con `localStorage`

## Ajustar el juego

Casi todo se puede balancear desde `src/constants.rs` sin tocar la lógica: velocidad de la torreta, vidas iniciales, probabilidad de power-ups, puntos por nivel, dificultad máxima, etc.

## Licencia

Proyecto educativo. Agrega aquí la licencia que prefieras (por ejemplo MIT).
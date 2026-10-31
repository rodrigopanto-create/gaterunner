# Gate Runner

Un juego 2D hecho en **Rust** con **[macroquad](https://macroquad.rs/)**, pensado como **proyecto educativo**: cada paso agrega una idea nueva y el código está comentado en español.

Combina la mecánica de las puertas de multiplicación (`x2`, `+5`, `-10`) con un *shoot 'em up* y un toque de *tower defense*: dispara a las oleadas de enemigos que bajan y evita que crucen tu línea.

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

```bash
git clone https://github.com/<tu-usuario>/gaterunner.git
cd gaterunner
cargo run --release
```

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

## Ideas para seguir

- Enemigo jefe cada 5 niveles, con barra de vida
- Distintos tipos de enemigos (rápidos, blindados, zigzag)
- Música que cambie con el nivel
- Pantalla de tabla de récords
- Controles táctiles o de gamepad

## Ajustar el juego

Casi todo se puede balancear desde `src/constants.rs` sin tocar la lógica: velocidad de la torreta, vidas iniciales, probabilidad de power-ups, puntos por nivel, dificultad máxima, etc.

## Licencia

Proyecto educativo. Agrega aquí la licencia que prefieras (por ejemplo MIT).
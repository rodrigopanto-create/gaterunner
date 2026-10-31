// ============================================================
// GATE RUNNER - Paso 11: codigo dividido en modulos
// ============================================================
// Antes todo vivia en este archivo (mas de 1300 lineas). Ahora cada
// archivo tiene UNA responsabilidad:
//
//   main.rs       -> ventana y loop principal (este archivo)
//   constants.rs  -> numeros de ajuste del juego
//   entities.rs   -> objetos: balas, enemigos, puertas, power-ups...
//   game.rs       -> estado y logica de la partida (struct Game)
//   game/render.rs-> dibujo en pantalla (hijo de game.rs)
//   highscore.rs  -> leer / guardar el record en un archivo
//   synth.rs      -> sintetizador de sonidos y creacion de WAV
//   audio.rs      -> efectos de sonido (Sfx) y reproductor (Sounds)
//   music.rs      -> melodia de fondo
//
// No cambia NADA de como se juega: solo donde esta cada cosa.
// ============================================================

// "mod nombre;" le dice a Rust que busque el archivo src/nombre.rs
// y lo trate como un modulo. Sin esta linea, el archivo se ignora.
mod audio;
mod constants;
mod entities;
mod game;
mod highscore;
mod music;
mod synth;

// "use" trae nombres de un modulo para usarlos sin escribir la ruta completa.
use audio::Sounds;
use game::Game;
use highscore::load_high_score;
use macroquad::prelude::*;
use music::volume_for_state;

// Configuracion de la ventana (tamano vertical, como un juego de celular)
fn window_conf() -> Conf {
    Conf {
        window_title: "Gate Runner".to_string(),
        window_width: 480,
        window_height: 800,
        window_resizable: false,
        ..Default::default()
    }
}

// main: crea los sonidos y la partida, y repite actualizar + sonar + dibujar.
#[macroquad::main(window_conf)]
async fn main() {
    // Semilla del generador de numeros aleatorios (la hora actual).
    rand::srand(macroquad::miniquad::date::now() as u64);

    // Creamos todos los sonidos una sola vez, al arrancar.
    let sounds = Sounds::load().await;

    // Cargamos el record guardado y arrancamos en el menu.
    let mut game = Game::new(load_high_score());

    // true = sonido silenciado (tecla N)
    let mut muted = false;

    // Arrancamos la musica en bucle (empieza en silencio).
    sounds.start_music();
    // Ultimo volumen aplicado. Empieza en -1.0 para forzar el primer ajuste.
    let mut last_music_volume: f32 = -1.0;

    loop {
        let dt = get_frame_time();

        if is_key_pressed(KeyCode::N) {
            muted = !muted; // alterna entre true y false
        }

        game.update(dt); // logica

        // Reproducimos los sonidos que la logica pidio en este frame.
        // take_sfx() siempre vacia la lista, incluso en silencio,
        // para que no se acumulen eventos.
        let events = game.take_sfx();
        if !muted {
            for sfx in events {
                sounds.play(sfx);
            }
        }

        // Volumen de la musica segun el estado (0 si esta silenciado).
        // Solo lo cambiamos cuando es distinto al anterior.
        let target_volume = if muted { 0.0 } else { volume_for_state(game.state) };
        if target_volume != last_music_volume {
            sounds.set_music_volume(target_volume);
            last_music_volume = target_volume;
        }

        game.draw(); // dibujo

        // Indicador de sonido, abajo a la derecha, encima de todo lo demas.
        let label = if muted { "N: sonido OFF" } else { "N: sonido ON" };
        draw_text(label, screen_width() - 130.0, screen_height() - 10.0, 18.0, GRAY);

        next_frame().await;
    }
}
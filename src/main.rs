// ============================================================
// GATE RUNNER - Paso 1: ventana y torreta que se mueve
// ============================================================
// Objetivo de este paso:
//   1. Abrir una ventana.
//   2. Dibujar nuestra torreta abajo.
//   3. Moverla con las flechas (o A / D).
// ============================================================

// "prelude" importa todo lo comun de macroquad (colores, teclas, dibujo...)
use macroquad::prelude::*;

// ---------- CONSTANTES ----------
// Las constantes nos permiten cambiar valores del juego en un solo lugar.
const PLAYER_SPEED: f32 = 300.0; // pixeles por segundo
const PLAYER_WIDTH: f32 = 50.0; // ancho de la torreta
const PLAYER_HEIGHT: f32 = 30.0; // alto de la torreta
const PLAYER_BOTTOM_MARGIN: f32 = 80.0; // distancia desde el borde inferior

// Configuracion de la ventana (tamano vertical, como un juego de celular)
fn window_conf() -> Conf {
    Conf {
        window_title: "Gate Runner".to_string(),
        window_width: 480,
        window_height: 800,
        window_resizable: false,
        ..Default::default() // el resto de opciones: valores por defecto
    }
}

// #[macroquad::main] prepara la ventana y nos deja usar "async" en main.
#[macroquad::main(window_conf)]
async fn main() {
    // Posicion horizontal del centro de la torreta.
    // "mut" significa que la variable puede cambiar.
    let mut player_x = screen_width() / 2.0;

    // ---------- GAME LOOP ----------
    // Cada vuelta del loop es un "frame" (un cuadro del juego).
    loop {
        // --- 1. ACTUALIZAR (logica) ---

        // Tiempo (en segundos) que paso desde el frame anterior.
        // Multiplicar por dt hace que el movimiento sea igual de rapido
        // en cualquier computadora, sin importar los FPS.
        let dt = get_frame_time();

        // Mover a la izquierda
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            player_x -= PLAYER_SPEED * dt;
        }
        // Mover a la derecha
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            player_x += PLAYER_SPEED * dt;
        }

        // "clamp" limita el valor entre un minimo y un maximo,
        // asi la torreta no se sale de la pantalla.
        player_x = player_x.clamp(
            PLAYER_WIDTH / 2.0,
            screen_width() - PLAYER_WIDTH / 2.0,
        );

        // --- 2. DIBUJAR ---

        // Pintamos el fondo (esto "borra" el frame anterior)
        clear_background(Color::from_rgba(20, 24, 40, 255));

        // Rectangulo: (x, y, ancho, alto, color).
        // x e y son la esquina superior izquierda, por eso
        // restamos la mitad del ancho para centrar la torreta en player_x.
        let player_y = screen_height() - PLAYER_BOTTOM_MARGIN;
        draw_rectangle(
            player_x - PLAYER_WIDTH / 2.0,
            player_y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            SKYBLUE,
        );

        // Texto de ayuda
        draw_text("Flechas o A/D para mover", 10.0, 30.0, 24.0, WHITE);

        // Espera al siguiente frame (siempre va al final del loop)
        next_frame().await;
    }
}
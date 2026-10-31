// ============================================================
// GATE RUNNER - Paso 2: disparar balas
// ============================================================
// Objetivo de este paso:
//   1. Crear un "struct" para representar una bala.
//   2. Guardar todas las balas en un Vec (lista que crece y se achica).
//   3. Disparar con ESPACIO, con un tiempo de espera entre disparos.
//   4. Borrar las balas que salen de la pantalla.
// ============================================================

// "prelude" importa todo lo comun de macroquad (colores, teclas, dibujo...)
use macroquad::prelude::*;

// ---------- CONSTANTES ----------
// Las constantes nos permiten cambiar valores del juego en un solo lugar.
const PLAYER_SPEED: f32 = 300.0; // pixeles por segundo
const PLAYER_WIDTH: f32 = 50.0; // ancho de la torreta
const PLAYER_HEIGHT: f32 = 30.0; // alto de la torreta
const PLAYER_BOTTOM_MARGIN: f32 = 80.0; // distancia desde el borde inferior

const BULLET_SPEED: f32 = 500.0; // pixeles por segundo (hacia arriba)
const BULLET_RADIUS: f32 = 5.0; // tamano de la bala
const FIRE_COOLDOWN: f32 = 0.25; // segundos de espera entre disparos

// ---------- STRUCTS ----------
// Un struct agrupa datos relacionados en un solo "tipo".
// Una bala solo necesita saber donde esta.
struct Bullet {
    x: f32,
    y: f32,
}

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

    // Lista de balas activas. Empieza vacia.
    // Vec<Bullet> = "vector (lista) de balas".
    let mut bullets: Vec<Bullet> = Vec::new();

    // Cronometro para limitar la velocidad de disparo.
    // Cuando llega a 0 (o menos) podemos disparar otra vez.
    let mut fire_timer: f32 = 0.0;

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

        // Posicion vertical de la torreta (la usamos para disparar y dibujar)
        let player_y = screen_height() - PLAYER_BOTTOM_MARGIN;

        // --- DISPARO ---

        // Restamos tiempo al cronometro en cada frame.
        fire_timer -= dt;

        // Si se presiona ESPACIO y ya paso el tiempo de espera...
        if is_key_down(KeyCode::Space) && fire_timer <= 0.0 {
            // ...creamos una bala justo en la punta de la torreta
            // y la agregamos a la lista con push().
            bullets.push(Bullet {
                x: player_x,
                y: player_y,
            });
            // Reiniciamos el cronometro.
            fire_timer = FIRE_COOLDOWN;
        }

        // --- MOVER BALAS ---

        // iter_mut() nos deja recorrer la lista Y modificar cada elemento.
        for bullet in bullets.iter_mut() {
            // En pantalla, "arriba" es Y menor, por eso restamos.
            bullet.y -= BULLET_SPEED * dt;
        }

        // retain() conserva solo las balas que cumplen la condicion.
        // Aqui borramos las que ya salieron por arriba de la pantalla,
        // para que la lista no crezca para siempre.
        bullets.retain(|bullet| bullet.y > -BULLET_RADIUS);

        // --- 2. DIBUJAR ---

        // Pintamos el fondo (esto "borra" el frame anterior)
        clear_background(Color::from_rgba(20, 24, 40, 255));

        // Dibujamos cada bala como un circulo: (x, y, radio, color)
        for bullet in &bullets {
            draw_circle(bullet.x, bullet.y, BULLET_RADIUS, YELLOW);
        }

        // Rectangulo: (x, y, ancho, alto, color).
        // x e y son la esquina superior izquierda, por eso
        // restamos la mitad del ancho para centrar la torreta en player_x.
        draw_rectangle(
            player_x - PLAYER_WIDTH / 2.0,
            player_y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            SKYBLUE,
        );

        // Texto de ayuda y contador de balas en pantalla (para depurar)
        draw_text("Flechas o A/D: mover | ESPACIO: disparar", 10.0, 30.0, 22.0, WHITE);
        draw_text(&format!("Balas: {}", bullets.len()), 10.0, 55.0, 22.0, GRAY);

        // Espera al siguiente frame (siempre va al final del loop)
        next_frame().await;
    }
}
// ============================================================
// GATE RUNNER - Paso 5: dificultad progresiva + struct Game
// ============================================================
// Objetivo de este paso:
//   1. Ordenar todo el estado del juego dentro de un "struct Game".
//   2. Dividir la logica en funciones pequenas (metodos).
//   3. Calcular un NIVEL segun el puntaje.
//   4. A mayor nivel: enemigos con mas vida, mas rapidos y mas frecuentes.
// ============================================================

// "prelude" importa todo lo comun de macroquad (colores, teclas, dibujo...)
use macroquad::prelude::*;

// ---------- CONSTANTES ----------
// Jugador
const PLAYER_SPEED: f32 = 300.0; // pixeles por segundo
const PLAYER_WIDTH: f32 = 50.0; // ancho de la torreta
const PLAYER_HEIGHT: f32 = 30.0; // alto de la torreta
const PLAYER_BOTTOM_MARGIN: f32 = 80.0; // distancia desde el borde inferior

// Balas
const BULLET_SPEED: f32 = 500.0; // pixeles por segundo (hacia arriba)
const BULLET_RADIUS: f32 = 5.0; // tamano de la bala
const FIRE_COOLDOWN: f32 = 0.25; // segundos de espera entre disparos
const MAX_SHOTS: i32 = 9; // maximo de balas por disparo
const BULLET_SPACING: f32 = 10.0; // separacion horizontal entre balas

// Enemigos (valores del NIVEL 1; suben con el nivel)
const ENEMY_SIZE: f32 = 40.0; // lado del cuadrado enemigo
const ENEMY_BASE_HP: i32 = 3; // vida inicial
const ENEMY_BASE_SPEED: f32 = 80.0; // velocidad inicial (pixeles por segundo)
const BASE_SPAWN_INTERVAL: f32 = 1.0; // segundos entre enemigos al inicio

// Limites de la dificultad (para que el juego no se vuelva imposible)
const ENEMY_MAX_SPEED: f32 = 220.0;
const MIN_SPAWN_INTERVAL: f32 = 0.35;

// Puntos necesarios para subir de nivel
const SCORE_PER_LEVEL: i32 = 100;

// Puertas
const GATE_HEIGHT: f32 = 50.0; // alto de cada puerta
const GATE_SPEED: f32 = 70.0; // pixeles por segundo (hacia abajo)
const GATE_INTERVAL: f32 = 5.0; // segundos entre filas de puertas

// Partida
const START_LIVES: i32 = 5;
const START_SOLDIERS: i32 = 1;
const MAX_SOLDIERS: i32 = 999;

// ---------- ENUM: tipos de puerta ----------
#[derive(Clone, Copy)]
enum GateOp {
    Add(i32),      // +N
    Subtract(i32), // -N
    Multiply(i32), // xN
}

impl GateOp {
    // Aplica la operacion a un numero de soldados.
    fn apply(&self, soldiers: i32) -> i32 {
        match self {
            GateOp::Add(n) => soldiers + n,
            GateOp::Subtract(n) => soldiers - n,
            GateOp::Multiply(n) => soldiers * n,
        }
    }

    // Texto que se muestra en la puerta.
    fn label(&self) -> String {
        match self {
            GateOp::Add(n) => format!("+{}", n),
            GateOp::Subtract(n) => format!("-{}", n),
            GateOp::Multiply(n) => format!("x{}", n),
        }
    }

    // Color de la puerta: verde/azul = bueno, rojo = malo.
    fn color(&self) -> Color {
        match self {
            GateOp::Add(_) => Color::new(0.2, 0.7, 0.3, 0.6),
            GateOp::Subtract(_) => Color::new(0.8, 0.2, 0.2, 0.6),
            GateOp::Multiply(_) => Color::new(0.2, 0.4, 0.9, 0.6),
        }
    }
}

// Crea una operacion de puerta al azar.
fn random_gate_op() -> GateOp {
    match rand::gen_range(0, 3) {
        0 => GateOp::Multiply(rand::gen_range(2, 4)), // x2 o x3
        1 => GateOp::Add(rand::gen_range(1, 11)),     // +1 a +10
        _ => GateOp::Subtract(rand::gen_range(1, 11)), // -1 a -10
    }
}

// ---------- STRUCTS: objetos del juego ----------
struct Bullet {
    x: f32,
    y: f32,
    alive: bool, // false = la bala ya golpeo algo y se borrara
}

struct Enemy {
    x: f32, // centro del enemigo
    y: f32,
    hp: i32,     // vida restante
    speed: f32,  // cada enemigo recuerda la velocidad con la que nacio
}

// Una fila de dos puertas (izquierda y derecha).
struct GatePair {
    y: f32,
    left: GateOp,
    right: GateOp,
    used: bool,
}

// Altura (Y) de la torreta. La usamos en varios lugares.
fn player_y() -> f32 {
    screen_height() - PLAYER_BOTTOM_MARGIN
}

// ---------- STRUCT GAME: todo el estado de la partida ----------
// Guardar todo junto nos permite reiniciar con una sola linea
// y dividir la logica en metodos pequenos.
struct Game {
    player_x: f32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    gates: Vec<GatePair>,
    fire_timer: f32,
    spawn_timer: f32,
    gate_timer: f32,
    score: i32,
    lives: i32,
    soldiers: i32,
    game_over: bool,
}

impl Game {
    // Crea una partida nueva con los valores iniciales.
    fn new() -> Game {
        Game {
            player_x: screen_width() / 2.0,
            bullets: Vec::new(),
            enemies: Vec::new(),
            gates: Vec::new(),
            fire_timer: 0.0,
            spawn_timer: 0.0,
            gate_timer: 2.0, // la primera fila de puertas sale a los 2 s
            score: 0,
            lives: START_LIVES,
            soldiers: START_SOLDIERS,
            game_over: false,
        }
    }

    // ---------- DIFICULTAD ----------
    // Nivel actual: empieza en 1 y sube cada SCORE_PER_LEVEL puntos.
    fn level(&self) -> i32 {
        self.score / SCORE_PER_LEVEL + 1
    }

    // Vida de los enemigos nuevos: +1 cada 2 niveles.
    fn enemy_hp(&self) -> i32 {
        ENEMY_BASE_HP + (self.level() - 1) / 2
    }

    // Velocidad de los enemigos nuevos: +10 por nivel, con un maximo.
    fn enemy_speed(&self) -> f32 {
        (ENEMY_BASE_SPEED + 10.0 * (self.level() - 1) as f32).min(ENEMY_MAX_SPEED)
    }

    // Tiempo entre enemigos: -0.07 s por nivel, con un minimo.
    fn spawn_interval(&self) -> f32 {
        (BASE_SPAWN_INTERVAL - 0.07 * (self.level() - 1) as f32).max(MIN_SPAWN_INTERVAL)
    }

    // ---------- ACTUALIZAR ----------
    // Se llama una vez por frame. Cada paso vive en su propio metodo.
    fn update(&mut self, dt: f32) {
        // En game over solo esperamos la tecla R para reiniciar.
        if self.game_over {
            if is_key_pressed(KeyCode::R) {
                // Reemplazamos todo el estado por una partida nueva.
                *self = Game::new();
            }
            return; // salimos: no se actualiza nada mas
        }

        self.handle_input(dt);
        self.spawn_things(dt);
        self.move_things(dt);
        self.check_collisions();
        self.cleanup();
    }

    // Teclado: mover y disparar.
    fn handle_input(&mut self, dt: f32) {
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.player_x -= PLAYER_SPEED * dt;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.player_x += PLAYER_SPEED * dt;
        }
        self.player_x = self
            .player_x
            .clamp(PLAYER_WIDTH / 2.0, screen_width() - PLAYER_WIDTH / 2.0);

        // Disparo con cooldown
        self.fire_timer -= dt;
        if is_key_down(KeyCode::Space) && self.fire_timer <= 0.0 {
            let shots = self.soldiers.clamp(1, MAX_SHOTS);
            for i in 0..shots {
                // Balas en fila, centradas en la torreta.
                let offset = (i as f32 - (shots - 1) as f32 / 2.0) * BULLET_SPACING;
                self.bullets.push(Bullet {
                    x: self.player_x + offset,
                    y: player_y(),
                    alive: true,
                });
            }
            self.fire_timer = FIRE_COOLDOWN;
        }
    }

    // Crea enemigos y puertas cuando sus cronometros llegan a 0.
    fn spawn_things(&mut self, dt: f32) {
        // Enemigos: su vida y velocidad dependen del nivel actual.
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            let x = rand::gen_range(ENEMY_SIZE / 2.0, screen_width() - ENEMY_SIZE / 2.0);
            self.enemies.push(Enemy {
                x,
                y: -ENEMY_SIZE,
                hp: self.enemy_hp(),
                speed: self.enemy_speed(),
            });
            self.spawn_timer = self.spawn_interval();
        }

        // Puertas
        self.gate_timer -= dt;
        if self.gate_timer <= 0.0 {
            self.gates.push(GatePair {
                y: -GATE_HEIGHT,
                left: random_gate_op(),
                right: random_gate_op(),
                used: false,
            });
            self.gate_timer = GATE_INTERVAL;
        }
    }

    // Mueve balas, enemigos y puertas. Tambien revisa si cruzamos una puerta.
    fn move_things(&mut self, dt: f32) {
        for bullet in self.bullets.iter_mut() {
            bullet.y -= BULLET_SPEED * dt;
        }
        for enemy in self.enemies.iter_mut() {
            enemy.y += enemy.speed * dt; // cada enemigo usa SU velocidad
        }

        for pair in self.gates.iter_mut() {
            pair.y += GATE_SPEED * dt;

            // ¿La fila esta a la altura de la torreta?
            let at_player_height =
                pair.y + GATE_HEIGHT >= player_y() && pair.y <= player_y() + PLAYER_HEIGHT;

            if at_player_height && !pair.used {
                // Mitad izquierda o derecha, segun donde este la torreta.
                let op = if self.player_x < screen_width() / 2.0 {
                    pair.left
                } else {
                    pair.right
                };
                self.soldiers = op.apply(self.soldiers).clamp(1, MAX_SOLDIERS);
                pair.used = true;
            }
        }
    }

    // Balas vs enemigos, puntos, vidas y game over.
    fn check_collisions(&mut self) {
        for bullet in self.bullets.iter_mut() {
            for enemy in self.enemies.iter_mut() {
                let enemy_rect = Rect::new(
                    enemy.x - ENEMY_SIZE / 2.0,
                    enemy.y - ENEMY_SIZE / 2.0,
                    ENEMY_SIZE,
                    ENEMY_SIZE,
                );
                if bullet.alive && enemy_rect.contains(vec2(bullet.x, bullet.y)) {
                    enemy.hp -= 1;
                    bullet.alive = false;
                    break;
                }
            }
        }

        // Enemigos destruidos -> +10 puntos cada uno
        let killed = self.enemies.iter().filter(|e| e.hp <= 0).count() as i32;
        self.score += killed * 10;

        // Enemigos vivos que cruzaron la linea inferior -> pierdes vidas
        let escaped = self
            .enemies
            .iter()
            .filter(|e| e.hp > 0 && e.y - ENEMY_SIZE / 2.0 > screen_height())
            .count() as i32;
        self.lives -= escaped;

        if self.lives <= 0 {
            self.lives = 0;
            self.game_over = true;
        }
    }

    // Borra lo que ya no sirve.
    fn cleanup(&mut self) {
        self.bullets.retain(|b| b.alive && b.y > -BULLET_RADIUS);
        self.enemies
            .retain(|e| e.hp > 0 && e.y - ENEMY_SIZE / 2.0 <= screen_height());
        self.gates.retain(|g| !g.used && g.y < screen_height());
    }

    // ---------- DIBUJAR ----------
    // &self (sin mut): dibujar solo LEE el estado, no lo cambia.
    fn draw(&self) {
        clear_background(Color::from_rgba(20, 24, 40, 255));

        // Puertas (primero, para que queden detras)
        let half = screen_width() / 2.0;
        for pair in &self.gates {
            let sides = [(0.0, pair.left), (half, pair.right)];
            for (gx, op) in sides {
                draw_rectangle(gx + 4.0, pair.y, half - 8.0, GATE_HEIGHT, op.color());

                let label = op.label();
                let size = measure_text(&label, None, 40, 1.0);
                draw_text(
                    &label,
                    gx + half / 2.0 - size.width / 2.0,
                    pair.y + GATE_HEIGHT / 2.0 + size.height / 2.0,
                    40.0,
                    WHITE,
                );
            }
        }

        // Balas
        for bullet in &self.bullets {
            draw_circle(bullet.x, bullet.y, BULLET_RADIUS, YELLOW);
        }

        // Enemigos
        for enemy in &self.enemies {
            draw_rectangle(
                enemy.x - ENEMY_SIZE / 2.0,
                enemy.y - ENEMY_SIZE / 2.0,
                ENEMY_SIZE,
                ENEMY_SIZE,
                RED,
            );
            draw_text(
                &enemy.hp.to_string(),
                enemy.x - 7.0,
                enemy.y + 8.0,
                28.0,
                WHITE,
            );
        }

        // Torreta y su numero de soldados
        draw_rectangle(
            self.player_x - PLAYER_WIDTH / 2.0,
            player_y(),
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            SKYBLUE,
        );
        draw_text(
            &format!("x{}", self.soldiers),
            self.player_x - 15.0,
            player_y() - 8.0,
            24.0,
            WHITE,
        );

        // Interfaz (HUD)
        draw_text(&format!("Nivel: {}", self.level()), 10.0, 30.0, 26.0, GOLD);
        draw_text(&format!("Puntos: {}", self.score), 10.0, 58.0, 26.0, WHITE);
        draw_text(&format!("Vidas: {}", self.lives), 10.0, 86.0, 26.0, ORANGE);
        draw_text(
            &format!("Soldados: {}", self.soldiers),
            10.0,
            114.0,
            26.0,
            SKYBLUE,
        );

        // Pantalla de game over
        if self.game_over {
            let title = "GAME OVER";
            let size = measure_text(title, None, 60, 1.0);
            draw_text(
                title,
                screen_width() / 2.0 - size.width / 2.0,
                screen_height() / 2.0,
                60.0,
                RED,
            );
            draw_text(
                "Presiona R para reiniciar",
                screen_width() / 2.0 - 140.0,
                screen_height() / 2.0 + 40.0,
                26.0,
                WHITE,
            );
        }
    }
}

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

// Ahora main es muy corto: crea la partida y repite actualizar + dibujar.
#[macroquad::main(window_conf)]
async fn main() {
    // Semilla del generador de numeros aleatorios (la hora actual).
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut game = Game::new();

    loop {
        let dt = get_frame_time();
        game.update(dt); // logica
        game.draw(); // dibujo
        next_frame().await;
    }
}
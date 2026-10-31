// ============================================================
// GATE RUNNER - Paso 7: estados del juego y puntaje maximo
// ============================================================
// Objetivo de este paso:
//   1. Un enum "State" para saber en que pantalla estamos:
//      Menu, Playing (jugando), Paused (pausa) o GameOver.
//   2. Pausar con P (o ESC) y volver con la misma tecla.
//   3. Guardar el record en un archivo (highscore.txt) para que
//      se conserve aunque cierres el juego.
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

// Limites de la dificultad
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

// Efectos visuales
const GRID_SIZE: f32 = 80.0; // distancia entre las lineas horizontales del fondo
const BG_SPEED: f32 = 60.0; // velocidad a la que bajan las lineas del fondo
const PARTICLES_PER_EXPLOSION: i32 = 12; // particulas al destruir un enemigo
const PARTICLE_LIFE: f32 = 0.5; // segundos que dura una particula
const BANNER_TIME: f32 = 2.0; // segundos que se ve el aviso de nivel

// Archivo donde se guarda el record (en la carpeta desde donde ejecutas el juego)
const HIGH_SCORE_FILE: &str = "highscore.txt";

// ---------- RECORD: leer y guardar en archivo ----------
// Lee el record guardado. Si el archivo no existe o esta danado, devuelve 0.
fn load_high_score() -> i32 {
    // read_to_string devuelve un "Result" (puede fallar). Con .ok() lo
    // convertimos en Option, con and_then/parse intentamos convertir el
    // texto a numero, y con unwrap_or(0) usamos 0 si algo fallo.
    std::fs::read_to_string(HIGH_SCORE_FILE)
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

// Guarda el record como texto en el archivo.
fn save_high_score(score: i32) {
    // "let _ =" ignora el resultado: si no se puede guardar, el juego sigue.
    let _ = std::fs::write(HIGH_SCORE_FILE, score.to_string());
}

// ---------- ENUM: estado del juego ----------
// El juego siempre esta en UNO de estos estados.
// PartialEq permite comparar con ==, Clone/Copy permiten copiarlo facil.
#[derive(Clone, Copy, PartialEq)]
enum State {
    Menu,     // pantalla de inicio
    Playing,  // jugando
    Paused,   // en pausa
    GameOver, // perdimos
}

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
    hp: i32,    // vida restante
    speed: f32, // cada enemigo recuerda la velocidad con la que nacio
}

// Una fila de dos puertas (izquierda y derecha).
struct GatePair {
    y: f32,
    left: GateOp,
    right: GateOp,
    used: bool,
}

// Una particula: un puntito que sale volando y se desvanece.
struct Particle {
    x: f32,
    y: f32,
    vx: f32,   // velocidad horizontal
    vy: f32,   // velocidad vertical
    life: f32, // segundos que le quedan; al llegar a 0 desaparece
}

// ---------- FUNCIONES AUXILIARES ----------
// Altura (Y) de la torreta. La usamos en varios lugares.
fn player_y() -> f32 {
    screen_height() - PLAYER_BOTTOM_MARGIN
}

// Dibuja un texto centrado horizontalmente en la pantalla.
// Asi no repetimos el calculo con measure_text cada vez.
fn draw_centered(text: &str, y: f32, size: u16, color: Color) {
    let dims = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - dims.width / 2.0,
        y,
        size as f32,
        color,
    );
}

// ---------- STRUCT GAME: todo el estado de la partida ----------
struct Game {
    state: State,      // NUEVO: en que pantalla estamos
    high_score: i32,   // NUEVO: mejor puntaje guardado
    new_record: bool,  // NUEVO: true si en esta partida superamos el record
    player_x: f32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    gates: Vec<GatePair>,
    particles: Vec<Particle>,
    fire_timer: f32,
    spawn_timer: f32,
    gate_timer: f32,
    score: i32,
    lives: i32,
    soldiers: i32,
    bg_scroll: f32,    // cuanto se ha desplazado el fondo
    last_level: i32,   // ultimo nivel visto (para detectar cuando sube)
    banner_timer: f32, // tiempo restante del aviso "NIVEL N"
}

impl Game {
    // Crea el juego en el menu. Recibe el record para no perderlo al reiniciar.
    fn new(high_score: i32) -> Game {
        Game {
            state: State::Menu,
            high_score,
            new_record: false,
            player_x: screen_width() / 2.0,
            bullets: Vec::new(),
            enemies: Vec::new(),
            gates: Vec::new(),
            particles: Vec::new(),
            fire_timer: 0.0,
            spawn_timer: 0.0,
            gate_timer: 2.0, // la primera fila de puertas sale a los 2 s
            score: 0,
            lives: START_LIVES,
            soldiers: START_SOLDIERS,
            bg_scroll: 0.0,
            last_level: 1,
            banner_timer: 0.0,
        }
    }

    // Empieza una partida nueva conservando el record.
    fn start_run(&mut self) {
        // Primero se calcula Game::new(...) y despues se reemplaza todo el estado.
        *self = Game::new(self.high_score);
        self.state = State::Playing;
    }

    // Se llama cuando las vidas llegan a 0.
    fn end_game(&mut self) {
        self.state = State::GameOver;

        // ¿Superamos el record? Lo actualizamos y lo guardamos en el archivo.
        if self.score > self.high_score {
            self.high_score = self.score;
            self.new_record = true;
            save_high_score(self.score);
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
    // Segun el estado actual, hacemos cosas distintas.
    fn update(&mut self, dt: f32) {
        // "match" revisa en que estado estamos y ejecuta solo esa rama.
        match self.state {
            State::Menu => {
                self.update_effects(dt); // el fondo sigue moviendose
                // ENTER empieza la partida (no usamos ESPACIO porque es el disparo).
                if is_key_pressed(KeyCode::Enter) {
                    self.start_run();
                }
            }

            State::Playing => {
                self.update_effects(dt);

                // P o ESC pausan el juego. "return" sale de update:
                // en ese frame ya no se mueve nada.
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
                    self.state = State::Paused;
                    return;
                }

                self.handle_input(dt);
                self.spawn_things(dt);
                self.move_things(dt);
                self.check_collisions();
                self.cleanup();
            }

            State::Paused => {
                // En pausa NO llamamos a update_effects ni a nada mas:
                // todo queda congelado. Solo esperamos para reanudar.
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
                    self.state = State::Playing;
                }
            }

            State::GameOver => {
                self.update_effects(dt);
                if is_key_pressed(KeyCode::R) {
                    self.start_run(); // jugar otra vez
                }
                if is_key_pressed(KeyCode::M) {
                    *self = Game::new(self.high_score); // volver al menu
                }
            }
        }
    }

    // Fondo, particulas y aviso de nivel.
    fn update_effects(&mut self, dt: f32) {
        // Fondo: avanza y vuelve a empezar cada GRID_SIZE pixeles.
        self.bg_scroll = (self.bg_scroll + BG_SPEED * dt) % GRID_SIZE;

        // Particulas: se mueven, envejecen y las muertas se borran.
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        // Aviso de nivel: si el nivel cambio, activamos el aviso.
        if self.level() != self.last_level {
            self.last_level = self.level();
            self.banner_timer = BANNER_TIME;
        }
        self.banner_timer = (self.banner_timer - dt).max(0.0);
    }

    // Crea una explosion de particulas en (x, y).
    fn spawn_explosion(&mut self, x: f32, y: f32) {
        for _ in 0..PARTICLES_PER_EXPLOSION {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(60.0, 200.0);

            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: PARTICLE_LIFE,
            });
        }
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

        // Posiciones de los enemigos destruidos (primero juntamos, luego usamos).
        let dead_positions: Vec<(f32, f32)> = self
            .enemies
            .iter()
            .filter(|e| e.hp <= 0)
            .map(|e| (e.x, e.y))
            .collect();

        // +10 puntos y una explosion por cada enemigo destruido
        for (x, y) in &dead_positions {
            self.score += 10;
            self.spawn_explosion(*x, *y);
        }

        // Enemigos vivos que cruzaron la linea inferior -> pierdes vidas
        let escaped = self
            .enemies
            .iter()
            .filter(|e| e.hp > 0 && e.y - ENEMY_SIZE / 2.0 > screen_height())
            .count() as i32;
        self.lives -= escaped;

        if self.lives <= 0 {
            self.lives = 0;
            self.end_game(); // cambia el estado a GameOver y revisa el record
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
        self.draw_background();

        // En el menu solo dibujamos la pantalla de inicio y terminamos.
        if self.state == State::Menu {
            self.draw_menu();
            return;
        }

        // ---- Desde aqui: Playing, Paused y GameOver comparten el dibujo del juego ----

        // Puertas (antes que lo demas, para que queden detras)
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

        // Particulas: se hacen mas transparentes y pequenas al envejecer.
        for p in &self.particles {
            let fraction = p.life / PARTICLE_LIFE; // 1.0 = recien nacida, 0.0 = por desaparecer
            let color = Color::new(1.0, 0.6, 0.1, fraction);
            draw_circle(p.x, p.y, 2.0 + 3.0 * fraction, color);
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
        // El record mostrado sube en vivo si lo estamos superando.
        let shown_record = self.high_score.max(self.score);
        draw_text(
            &format!("Record: {}", shown_record),
            screen_width() - 160.0,
            30.0,
            26.0,
            GREEN,
        );

        // Aviso "NIVEL N" (solo mientras banner_timer > 0)
        if self.banner_timer > 0.0 {
            let alpha = (self.banner_timer / BANNER_TIME).min(1.0); // se desvanece
            draw_centered(
                &format!("NIVEL {}", self.level()),
                screen_height() * 0.3,
                70,
                Color::new(1.0, 0.84, 0.0, alpha),
            );
        }

        // ---- Capas encima del juego segun el estado ----
        match self.state {
            State::Paused => {
                // Rectangulo negro semitransparente que oscurece la pantalla
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, 0.55),
                );
                draw_centered("PAUSA", screen_height() / 2.0, 70, WHITE);
                draw_centered(
                    "Presiona P para continuar",
                    screen_height() / 2.0 + 40.0,
                    26,
                    LIGHTGRAY,
                );
            }
            State::GameOver => {
                draw_centered("GAME OVER", screen_height() / 2.0 - 40.0, 60, RED);
                draw_centered(
                    &format!("Puntos: {}", self.score),
                    screen_height() / 2.0,
                    30,
                    WHITE,
                );
                // Mensaje de record: uno u otro segun si lo superamos.
                if self.new_record {
                    draw_centered("NUEVO RECORD!", screen_height() / 2.0 + 35.0, 36, GREEN);
                } else {
                    draw_centered(
                        &format!("Record: {}", self.high_score),
                        screen_height() / 2.0 + 35.0,
                        28,
                        LIGHTGRAY,
                    );
                }
                draw_centered(
                    "R: jugar otra vez   M: menu",
                    screen_height() / 2.0 + 80.0,
                    26,
                    WHITE,
                );
            }
            // Playing y Menu no dibujan nada extra aqui.
            _ => {}
        }
    }

    // Pantalla de inicio.
    fn draw_menu(&self) {
        draw_centered("GATE RUNNER", screen_height() * 0.28, 70, GOLD);
        draw_centered(
            "Cruza las puertas, multiplica",
            screen_height() * 0.28 + 40.0,
            26,
            SKYBLUE,
        );
        draw_centered(
            "tu ejercito y defiende la linea!",
            screen_height() * 0.28 + 70.0,
            26,
            SKYBLUE,
        );

        // Controles
        draw_centered("Mover: flechas o A / D", screen_height() * 0.52, 26, WHITE);
        draw_centered("Disparar: ESPACIO", screen_height() * 0.52 + 32.0, 26, WHITE);
        draw_centered("Pausa: P o ESC", screen_height() * 0.52 + 64.0, 26, WHITE);

        draw_centered(
            &format!("Record: {}", self.high_score),
            screen_height() * 0.52 + 120.0,
            30,
            GREEN,
        );

        // Texto parpadeante: get_time() son los segundos desde que arranco el juego.
        // Dividimos en "medios segundos" y mostramos el texto solo en los pares.
        if (get_time() * 2.0) as i32 % 2 == 0 {
            draw_centered(
                "Presiona ENTER para jugar",
                screen_height() * 0.85,
                30,
                WHITE,
            );
        }
    }

    // Dibuja el fondo (lineas horizontales que bajan + linea central).
    fn draw_background(&self) {
        let line_color = Color::from_rgba(255, 255, 255, 18); // blanco muy tenue

        let rows = (screen_height() / GRID_SIZE) as i32 + 2;
        for i in -1..rows {
            let y = i as f32 * GRID_SIZE + self.bg_scroll;
            draw_line(0.0, y, screen_width(), y, 1.0, line_color);
        }

        // Linea central: separa visualmente la puerta izquierda de la derecha.
        let mid = screen_width() / 2.0;
        draw_line(mid, 0.0, mid, screen_height(), 2.0, line_color);
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

// main sigue siendo muy corto: crea la partida y repite actualizar + dibujar.
#[macroquad::main(window_conf)]
async fn main() {
    // Semilla del generador de numeros aleatorios (la hora actual).
    rand::srand(macroquad::miniquad::date::now() as u64);

    // Cargamos el record guardado y arrancamos en el menu.
    let mut game = Game::new(load_high_score());

    loop {
        let dt = get_frame_time();
        game.update(dt); // logica
        game.draw(); // dibujo
        next_frame().await;
    }
}
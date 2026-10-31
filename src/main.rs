// ============================================================
// GATE RUNNER - Paso 8: power-ups y fondo de estrellas
// ============================================================
// Objetivo de este paso:
//   1. Los enemigos destruidos a veces sueltan un power-up que cae.
//      Al tocarlo con la torreta se activa:
//        R = disparo rapido (por unos segundos)
//        S = escudo (bloquea el siguiente enemigo que se escape)
//        + = una vida extra
//   2. Fondo de estrellas con "parallax": las rapidas se ven
//      grandes y brillantes (cerca), las lentas pequenas (lejos).
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
const MAX_LIVES: i32 = 9; // con items de vida no pasamos de aqui
const START_SOLDIERS: i32 = 1;
const MAX_SOLDIERS: i32 = 999;

// Power-ups
const POWERUP_DROP_CHANCE: f32 = 0.2; // 0.2 = 20% de probabilidad al destruir un enemigo
const POWERUP_SPEED: f32 = 110.0; // velocidad de caida
const POWERUP_RADIUS: f32 = 14.0; // tamano del circulo
const RAPID_DURATION: f32 = 6.0; // segundos que dura el disparo rapido
const RAPID_FIRE_FACTOR: f32 = 0.4; // el cooldown se multiplica por esto (menos = mas rapido)

// Fondo de estrellas
const STAR_COUNT: i32 = 90; // cuantas estrellas hay
const STAR_MIN_SPEED: f32 = 20.0; // estrella mas lenta (lejana)
const STAR_MAX_SPEED: f32 = 150.0; // estrella mas rapida (cercana)

// Efectos visuales
const PARTICLES_PER_EXPLOSION: i32 = 12; // particulas al destruir un enemigo
const PARTICLE_LIFE: f32 = 0.5; // segundos que dura una particula
const BANNER_TIME: f32 = 2.0; // segundos que se ve el aviso de nivel

// Archivo donde se guarda el record (en la carpeta desde donde ejecutas el juego)
const HIGH_SCORE_FILE: &str = "highscore.txt";

// ---------- RECORD: leer y guardar en archivo ----------
// Lee el record guardado. Si el archivo no existe o esta danado, devuelve 0.
fn load_high_score() -> i32 {
    std::fs::read_to_string(HIGH_SCORE_FILE)
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

// Guarda el record como texto en el archivo.
fn save_high_score(score: i32) {
    let _ = std::fs::write(HIGH_SCORE_FILE, score.to_string());
}

// ---------- ENUM: estado del juego ----------
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

// ---------- ENUM: tipos de power-up ----------
#[derive(Clone, Copy)]
enum PowerUpKind {
    RapidFire, // disparo rapido por unos segundos
    Shield,    // bloquea el siguiente enemigo que se escape
    Heart,     // +1 vida
}

impl PowerUpKind {
    // Color del circulo.
    fn color(&self) -> Color {
        match self {
            PowerUpKind::RapidFire => ORANGE,
            PowerUpKind::Shield => SKYBLUE,
            PowerUpKind::Heart => GREEN,
        }
    }

    // Letra que se dibuja dentro del circulo.
    fn letter(&self) -> &'static str {
        match self {
            PowerUpKind::RapidFire => "R",
            PowerUpKind::Shield => "S",
            PowerUpKind::Heart => "+",
        }
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

// Un power-up que cae esperando a que lo recojamos.
struct PowerUp {
    x: f32,
    y: f32,
    kind: PowerUpKind,
}

// Una estrella del fondo. Su velocidad decide cuan "cerca" parece.
struct Star {
    x: f32,
    y: f32,
    speed: f32,
}

// ---------- FUNCIONES AUXILIARES ----------
// Altura (Y) de la torreta. La usamos en varios lugares.
fn player_y() -> f32 {
    screen_height() - PLAYER_BOTTOM_MARGIN
}

// Dibuja un texto centrado horizontalmente en la pantalla.
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

// Crea todas las estrellas del fondo en posiciones y velocidades al azar.
fn new_stars() -> Vec<Star> {
    let mut stars = Vec::new();
    for _ in 0..STAR_COUNT {
        stars.push(Star {
            x: rand::gen_range(0.0, screen_width()),
            y: rand::gen_range(0.0, screen_height()),
            speed: rand::gen_range(STAR_MIN_SPEED, STAR_MAX_SPEED),
        });
    }
    stars
}

// ---------- STRUCT GAME: todo el estado de la partida ----------
struct Game {
    state: State,     // en que pantalla estamos
    high_score: i32,  // mejor puntaje guardado
    new_record: bool, // true si en esta partida superamos el record
    player_x: f32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    gates: Vec<GatePair>,
    particles: Vec<Particle>,
    powerups: Vec<PowerUp>, // NUEVO: power-ups cayendo
    stars: Vec<Star>,       // NUEVO: estrellas del fondo
    rapid_timer: f32,       // NUEVO: segundos restantes de disparo rapido
    shield: bool,           // NUEVO: true si tenemos escudo activo
    fire_timer: f32,
    spawn_timer: f32,
    gate_timer: f32,
    score: i32,
    lives: i32,
    soldiers: i32,
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
            powerups: Vec::new(),
            stars: new_stars(),
            rapid_timer: 0.0,
            shield: false,
            fire_timer: 0.0,
            spawn_timer: 0.0,
            gate_timer: 2.0, // la primera fila de puertas sale a los 2 s
            score: 0,
            lives: START_LIVES,
            soldiers: START_SOLDIERS,
            last_level: 1,
            banner_timer: 0.0,
        }
    }

    // Reinicia todo y pasa al estado indicado (Playing o Menu).
    // Conservamos el record y las estrellas para que el fondo no "salte".
    fn restart(&mut self, new_state: State) {
        let high_score = self.high_score;
        // std::mem::take saca las estrellas de self y deja un Vec vacio en su lugar.
        let stars = std::mem::take(&mut self.stars);

        *self = Game::new(high_score); // partida nueva...
        self.stars = stars; // ...pero con las mismas estrellas
        self.state = new_state;
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
        match self.state {
            State::Menu => {
                self.update_effects(dt); // las estrellas siguen moviendose
                if is_key_pressed(KeyCode::Enter) {
                    self.restart(State::Playing);
                }
            }

            State::Playing => {
                self.update_effects(dt);

                // P o ESC pausan el juego.
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
                    self.state = State::Paused;
                    return;
                }

                self.handle_input(dt);
                self.spawn_things(dt);
                self.move_things(dt);
                self.collect_powerups(); // NUEVO
                self.check_collisions();
                self.cleanup();
            }

            State::Paused => {
                // En pausa todo queda congelado. Solo esperamos para reanudar.
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
                    self.state = State::Playing;
                }
            }

            State::GameOver => {
                self.update_effects(dt);
                if is_key_pressed(KeyCode::R) {
                    self.restart(State::Playing); // jugar otra vez
                }
                if is_key_pressed(KeyCode::M) {
                    self.restart(State::Menu); // volver al menu
                }
            }
        }
    }

    // Estrellas, particulas y aviso de nivel.
    fn update_effects(&mut self, dt: f32) {
        // Estrellas: bajan a su propia velocidad. Al salir por abajo,
        // reaparecen arriba en una posicion horizontal nueva.
        for star in self.stars.iter_mut() {
            star.y += star.speed * dt;
            if star.y > screen_height() {
                star.y = 0.0;
                star.x = rand::gen_range(0.0, screen_width());
            }
        }

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

    // NUEVO: con cierta probabilidad, suelta un power-up en (x, y).
    fn maybe_drop_powerup(&mut self, x: f32, y: f32) {
        // gen_range(0.0, 1.0) da un decimal entre 0 y 1.
        // Si sale menor que 0.2, ocurre el 20% de las veces.
        if rand::gen_range(0.0, 1.0) < POWERUP_DROP_CHANCE {
            // Elegimos el tipo al azar.
            let kind = match rand::gen_range(0, 3) {
                0 => PowerUpKind::RapidFire,
                1 => PowerUpKind::Shield,
                _ => PowerUpKind::Heart,
            };
            self.powerups.push(PowerUp { x, y, kind });
        }
    }

    // NUEVO: aplica el efecto de un power-up recogido.
    fn apply_powerup(&mut self, kind: PowerUpKind) {
        match kind {
            PowerUpKind::RapidFire => self.rapid_timer = RAPID_DURATION,
            PowerUpKind::Shield => self.shield = true,
            PowerUpKind::Heart => self.lives = (self.lives + 1).min(MAX_LIVES),
        }
    }

    // NUEVO: revisa si la torreta toca algun power-up y lo recoge.
    fn collect_powerups(&mut self) {
        // Rectangulo de la torreta, un poco agrandado por el radio del item.
        let player_rect = Rect::new(
            self.player_x - PLAYER_WIDTH / 2.0 - POWERUP_RADIUS,
            player_y() - POWERUP_RADIUS,
            PLAYER_WIDTH + 2.0 * POWERUP_RADIUS,
            PLAYER_HEIGHT + 2.0 * POWERUP_RADIUS,
        );

        // Aqui guardamos los tipos recogidos en este frame.
        let mut picked: Vec<PowerUpKind> = Vec::new();

        // retain conserva los elementos para los que la funcion devuelve true.
        // Aprovechamos para: si lo tocamos -> lo anotamos y lo borramos (false);
        // si no -> lo conservamos mientras no haya salido de la pantalla.
        self.powerups.retain(|p| {
            if player_rect.contains(vec2(p.x, p.y)) {
                picked.push(p.kind);
                false
            } else {
                p.y - POWERUP_RADIUS < screen_height()
            }
        });

        // Aplicamos los efectos (fuera del retain para no prestar self dos veces).
        for kind in picked {
            self.apply_powerup(kind);
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

        // El disparo rapido se va acabando con el tiempo (sin bajar de 0).
        self.rapid_timer = (self.rapid_timer - dt).max(0.0);

        // Si el disparo rapido esta activo, el cooldown es mas corto.
        let cooldown = if self.rapid_timer > 0.0 {
            FIRE_COOLDOWN * RAPID_FIRE_FACTOR
        } else {
            FIRE_COOLDOWN
        };

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
            self.fire_timer = cooldown;
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

    // Mueve balas, enemigos, power-ups y puertas. Tambien revisa si cruzamos una puerta.
    fn move_things(&mut self, dt: f32) {
        for bullet in self.bullets.iter_mut() {
            bullet.y -= BULLET_SPEED * dt;
        }
        for enemy in self.enemies.iter_mut() {
            enemy.y += enemy.speed * dt; // cada enemigo usa SU velocidad
        }
        for powerup in self.powerups.iter_mut() {
            powerup.y += POWERUP_SPEED * dt;
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

        // Por cada enemigo destruido: +10 puntos, explosion y quiza un power-up.
        for (x, y) in &dead_positions {
            self.score += 10;
            self.spawn_explosion(*x, *y);
            self.maybe_drop_powerup(*x, *y);
        }

        // Enemigos vivos que cruzaron la linea inferior
        let mut escaped = self
            .enemies
            .iter()
            .filter(|e| e.hp > 0 && e.y - ENEMY_SIZE / 2.0 > screen_height())
            .count() as i32;

        // NUEVO: el escudo absorbe UN enemigo escapado y se consume.
        if self.shield && escaped > 0 {
            self.shield = false;
            escaped -= 1;
        }

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
        clear_background(Color::from_rgba(10, 12, 28, 255));
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

        // Power-ups: circulo de color con borde blanco y su letra al centro.
        for p in &self.powerups {
            draw_circle(p.x, p.y, POWERUP_RADIUS, p.kind.color());
            draw_circle_lines(p.x, p.y, POWERUP_RADIUS, 2.0, WHITE);

            let letter = p.kind.letter();
            let size = measure_text(letter, None, 22, 1.0);
            draw_text(
                letter,
                p.x - size.width / 2.0,
                p.y + size.height / 2.0,
                22.0,
                BLACK,
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

        // NUEVO: anillo alrededor de la torreta si tenemos escudo
        if self.shield {
            draw_circle_lines(
                self.player_x,
                player_y() + PLAYER_HEIGHT / 2.0,
                38.0,
                3.0,
                SKYBLUE,
            );
        }

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

        // NUEVO: efectos activos (cada uno ocupa una linea mas abajo)
        let mut effect_y = 142.0;
        if self.rapid_timer > 0.0 {
            draw_text(
                &format!("Disparo rapido: {:.0}s", self.rapid_timer.ceil()),
                10.0,
                effect_y,
                22.0,
                ORANGE,
            );
            effect_y += 26.0;
        }
        if self.shield {
            draw_text("Escudo activo", 10.0, effect_y, 22.0, SKYBLUE);
        }

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

        // Leyenda de los power-ups
        draw_centered(
            "Items: R rapido   S escudo   + vida",
            screen_height() * 0.52 + 104.0,
            22,
            LIGHTGRAY,
        );

        draw_centered(
            &format!("Record: {}", self.high_score),
            screen_height() * 0.52 + 150.0,
            30,
            GREEN,
        );

        // Texto parpadeante: get_time() son los segundos desde que arranco el juego.
        if (get_time() * 2.0) as i32 % 2 == 0 {
            draw_centered(
                "Presiona ENTER para jugar",
                screen_height() * 0.85,
                30,
                WHITE,
            );
        }
    }

    // NUEVO: dibuja el fondo de estrellas.
    fn draw_background(&self) {
        for star in &self.stars {
            // t va de 0.0 (estrella mas lenta) a 1.0 (la mas rapida).
            let t = (star.speed - STAR_MIN_SPEED) / (STAR_MAX_SPEED - STAR_MIN_SPEED);

            // Parallax: las rapidas parecen cercanas -> mas grandes y brillantes.
            let size = 1.0 + 1.5 * t; // radio de 1.0 a 2.5
            let alpha = 0.25 + 0.65 * t; // transparencia de 0.25 a 0.9

            draw_circle(star.x, star.y, size, Color::new(0.8, 0.9, 1.0, alpha));
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
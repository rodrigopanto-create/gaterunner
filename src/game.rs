// ============================================================
// game.rs - Estado y logica de la partida
// ============================================================
// Aqui vive el struct Game con todo lo que cambia mientras se juega:
// moverse, disparar, enemigos, puertas, colisiones, puntos...
//
// El DIBUJO esta en game/render.rs. Lo declaramos aqui como "modulo hijo"
// (mod render) para que pueda leer los campos privados de Game.
// ============================================================

use macroquad::prelude::*;

use crate::audio::Sfx;
use crate::constants::*;
use crate::entities::*;
use crate::highscore::save_high_score;

// Modulo hijo: el archivo es src/game/render.rs
mod render;

// ---------- ENUM: estado del juego ----------
#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Menu,     // pantalla de inicio
    Playing,  // jugando
    Paused,   // en pausa
    GameOver, // perdimos
}

// Altura (Y) de la torreta. La usan game.rs y render.rs.
fn player_y() -> f32 {
    screen_height() - PLAYER_BOTTOM_MARGIN
}

// ---------- STRUCT GAME: todo el estado de la partida ----------
// Solo "state" es pub (main.rs lo necesita para ajustar la musica).
// El resto es privado: nadie fuera de este modulo lo toca por accidente.
pub struct Game {
    pub state: State, // en que pantalla estamos
    high_score: i32,  // mejor puntaje guardado
    new_record: bool, // true si en esta partida superamos el record
    player_x: f32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    gates: Vec<GatePair>,
    particles: Vec<Particle>,
    powerups: Vec<PowerUp>, // power-ups cayendo
    stars: Vec<Star>,       // estrellas del fondo
    sfx: Vec<Sfx>,          // eventos de sonido pendientes de reproducir
    rapid_timer: f32,       // segundos restantes de disparo rapido
    shield: bool,           // true si tenemos escudo activo
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
    pub fn new(high_score: i32) -> Game {
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
            sfx: Vec::new(),
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

    // Entrega los eventos de sonido pendientes y deja la lista vacia.
    // main los usa para reproducirlos.
    pub fn take_sfx(&mut self) -> Vec<Sfx> {
        std::mem::take(&mut self.sfx)
    }

    // Reinicia todo y pasa al estado indicado (Playing o Menu).
    // Conservamos el record y las estrellas para que el fondo no "salte".
    fn restart(&mut self, new_state: State) {
        let high_score = self.high_score;
        let stars = std::mem::take(&mut self.stars);

        *self = Game::new(high_score); // partida nueva...
        self.stars = stars; // ...pero con las mismas estrellas
        self.state = new_state;
    }

    // Se llama cuando las vidas llegan a 0.
    fn end_game(&mut self) {
        self.state = State::GameOver;
        self.sfx.push(Sfx::GameOver);

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
    pub fn update(&mut self, dt: f32) {
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
                self.collect_powerups();
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

        // Aviso de nivel: si el nivel cambio, activamos el aviso y el sonido.
        if self.level() != self.last_level {
            self.last_level = self.level();
            self.banner_timer = BANNER_TIME;
            self.sfx.push(Sfx::LevelUp);
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

    // Con cierta probabilidad, suelta un power-up en (x, y).
    fn maybe_drop_powerup(&mut self, x: f32, y: f32) {
        if rand::gen_range(0.0, 1.0) < POWERUP_DROP_CHANCE {
            let kind = match rand::gen_range(0, 3) {
                0 => PowerUpKind::RapidFire,
                1 => PowerUpKind::Shield,
                _ => PowerUpKind::Heart,
            };
            self.powerups.push(PowerUp { x, y, kind });
        }
    }

    // Aplica el efecto de un power-up recogido.
    fn apply_powerup(&mut self, kind: PowerUpKind) {
        match kind {
            PowerUpKind::RapidFire => self.rapid_timer = RAPID_DURATION,
            PowerUpKind::Shield => self.shield = true,
            PowerUpKind::Heart => self.lives = (self.lives + 1).min(MAX_LIVES),
        }
    }

    // Revisa si la torreta toca algun power-up y lo recoge.
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
            self.sfx.push(Sfx::PowerUp);
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
            // Un solo sonido por disparo, sin importar cuantas balas salgan.
            self.sfx.push(Sfx::Shoot);
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

                // Sonido segun el tipo de puerta. matches! devuelve true si
                // "op" es de la variante indicada (el _ ignora el numero).
                if matches!(op, GateOp::Subtract(_)) {
                    self.sfx.push(Sfx::GateBad);
                } else {
                    self.sfx.push(Sfx::GateGood);
                }
            }
        }
    }

    // Balas vs enemigos, puntos, vidas y game over.
    fn check_collisions(&mut self) {
        // Para tocar UN solo sonido de golpe por frame (aunque haya muchas balas).
        let mut any_hit = false;

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
                    any_hit = true;
                    break;
                }
            }
        }

        if any_hit {
            self.sfx.push(Sfx::Hit);
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
        if !dead_positions.is_empty() {
            self.sfx.push(Sfx::Explosion); // un solo sonido aunque mueran varios
        }

        // Enemigos vivos que cruzaron la linea inferior
        let mut escaped = self
            .enemies
            .iter()
            .filter(|e| e.hp > 0 && e.y - ENEMY_SIZE / 2.0 > screen_height())
            .count() as i32;

        // El escudo absorbe UN enemigo escapado y se consume.
        if self.shield && escaped > 0 {
            self.shield = false;
            escaped -= 1;
            self.sfx.push(Sfx::ShieldBreak);
        }

        self.lives -= escaped;

        if self.lives <= 0 {
            self.lives = 0;
            self.end_game(); // cambia el estado a GameOver (y suena GameOver)
        } else if escaped > 0 {
            self.sfx.push(Sfx::LifeLost);
        }
    }

    // Borra lo que ya no sirve.
    fn cleanup(&mut self) {
        self.bullets.retain(|b| b.alive && b.y > -BULLET_RADIUS);
        self.enemies
            .retain(|e| e.hp > 0 && e.y - ENEMY_SIZE / 2.0 <= screen_height());
        self.gates.retain(|g| !g.used && g.y < screen_height());
    }
}
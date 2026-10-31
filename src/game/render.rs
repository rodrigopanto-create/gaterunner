// ============================================================
// game/render.rs - Dibujo del juego
// ============================================================
// Todo lo que se ve en pantalla: fondo, objetos, HUD, menu y capas
// de pausa / game over. Solo LEE el estado de Game, nunca lo cambia.
//
// Este archivo es un "modulo hijo" de game.rs, por eso puede leer los
// campos privados de Game con self.campo.
// ============================================================

use macroquad::prelude::*;

use super::{player_y, Game, State};
use crate::constants::*;

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

// Podemos tener varios bloques "impl Game" en archivos distintos.
impl Game {
    // ---------- DIBUJAR ----------
    // &self (sin mut): dibujar solo LEE el estado, no lo cambia.
    pub fn draw(&self) {
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

        // Anillo alrededor de la torreta si tenemos escudo
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

        // Efectos activos (cada uno ocupa una linea mas abajo)
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

    // Dibuja el fondo de estrellas.
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
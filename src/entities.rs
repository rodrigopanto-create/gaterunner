// ============================================================
// entities.rs - Los objetos del juego (datos, sin logica de partida)
// ============================================================
// Balas, enemigos, puertas, particulas, power-ups y estrellas.
// Los campos son "pub" para que game.rs y render.rs puedan leerlos.
// ============================================================

use macroquad::prelude::*;

use crate::constants::*;

// ---------- ENUM: tipos de puerta ----------
#[derive(Clone, Copy)]
pub enum GateOp {
    Add(i32),      // +N
    Subtract(i32), // -N
    Multiply(i32), // xN
}

impl GateOp {
    // Aplica la operacion a un numero de soldados.
    pub fn apply(&self, soldiers: i32) -> i32 {
        match self {
            GateOp::Add(n) => soldiers + n,
            GateOp::Subtract(n) => soldiers - n,
            GateOp::Multiply(n) => soldiers * n,
        }
    }

    // Texto que se muestra en la puerta.
    pub fn label(&self) -> String {
        match self {
            GateOp::Add(n) => format!("+{}", n),
            GateOp::Subtract(n) => format!("-{}", n),
            GateOp::Multiply(n) => format!("x{}", n),
        }
    }

    // Color de la puerta: verde/azul = bueno, rojo = malo.
    pub fn color(&self) -> Color {
        match self {
            GateOp::Add(_) => Color::new(0.2, 0.7, 0.3, 0.6),
            GateOp::Subtract(_) => Color::new(0.8, 0.2, 0.2, 0.6),
            GateOp::Multiply(_) => Color::new(0.2, 0.4, 0.9, 0.6),
        }
    }
}

// Crea una operacion de puerta al azar.
pub fn random_gate_op() -> GateOp {
    match rand::gen_range(0, 3) {
        0 => GateOp::Multiply(rand::gen_range(2, 4)), // x2 o x3
        1 => GateOp::Add(rand::gen_range(1, 11)),     // +1 a +10
        _ => GateOp::Subtract(rand::gen_range(1, 11)), // -1 a -10
    }
}

// ---------- ENUM: tipos de power-up ----------
#[derive(Clone, Copy)]
pub enum PowerUpKind {
    RapidFire, // disparo rapido por unos segundos
    Shield,    // bloquea el siguiente enemigo que se escape
    Heart,     // +1 vida
}

impl PowerUpKind {
    // Color del circulo.
    pub fn color(&self) -> Color {
        match self {
            PowerUpKind::RapidFire => ORANGE,
            PowerUpKind::Shield => SKYBLUE,
            PowerUpKind::Heart => GREEN,
        }
    }

    // Letra que se dibuja dentro del circulo.
    pub fn letter(&self) -> &'static str {
        match self {
            PowerUpKind::RapidFire => "R",
            PowerUpKind::Shield => "S",
            PowerUpKind::Heart => "+",
        }
    }
}

// ---------- STRUCTS: objetos del juego ----------
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub alive: bool, // false = la bala ya golpeo algo y se borrara
}

pub struct Enemy {
    pub x: f32, // centro del enemigo
    pub y: f32,
    pub hp: i32,    // vida restante
    pub speed: f32, // cada enemigo recuerda la velocidad con la que nacio
}

// Una fila de dos puertas (izquierda y derecha).
pub struct GatePair {
    pub y: f32,
    pub left: GateOp,
    pub right: GateOp,
    pub used: bool,
}

// Una particula: un puntito que sale volando y se desvanece.
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,   // velocidad horizontal
    pub vy: f32,   // velocidad vertical
    pub life: f32, // segundos que le quedan; al llegar a 0 desaparece
}

// Un power-up que cae esperando a que lo recojamos.
pub struct PowerUp {
    pub x: f32,
    pub y: f32,
    pub kind: PowerUpKind,
}

// Una estrella del fondo. Su velocidad decide cuan "cerca" parece.
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
}

// Crea todas las estrellas del fondo en posiciones y velocidades al azar.
pub fn new_stars() -> Vec<Star> {
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
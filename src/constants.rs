// ============================================================
// constants.rs - Valores de ajuste del juego
// ============================================================
// Aqui estan los numeros que definen como se siente el juego.
// Cambia cualquiera para balancear la dificultad sin tocar la logica.
//
// "pub" significa "publico": los otros archivos (modulos) pueden usarlo.
// ============================================================

// ---------- Jugador ----------
pub const PLAYER_SPEED: f32 = 300.0; // pixeles por segundo
pub const PLAYER_WIDTH: f32 = 50.0; // ancho de la torreta
pub const PLAYER_HEIGHT: f32 = 30.0; // alto de la torreta
pub const PLAYER_BOTTOM_MARGIN: f32 = 80.0; // distancia desde el borde inferior

// ---------- Balas ----------
pub const BULLET_SPEED: f32 = 500.0; // pixeles por segundo (hacia arriba)
pub const BULLET_RADIUS: f32 = 5.0; // tamano de la bala
pub const FIRE_COOLDOWN: f32 = 0.25; // segundos de espera entre disparos
pub const MAX_SHOTS: i32 = 9; // maximo de balas por disparo
pub const BULLET_SPACING: f32 = 10.0; // separacion horizontal entre balas

// ---------- Enemigos (valores del NIVEL 1; suben con el nivel) ----------
pub const ENEMY_SIZE: f32 = 40.0; // lado del cuadrado enemigo
pub const ENEMY_BASE_HP: i32 = 3; // vida inicial
pub const ENEMY_BASE_SPEED: f32 = 80.0; // velocidad inicial (pixeles por segundo)
pub const BASE_SPAWN_INTERVAL: f32 = 1.0; // segundos entre enemigos al inicio

// ---------- Limites de la dificultad ----------
pub const ENEMY_MAX_SPEED: f32 = 220.0;
pub const MIN_SPAWN_INTERVAL: f32 = 0.35;

// Puntos necesarios para subir de nivel
pub const SCORE_PER_LEVEL: i32 = 100;

// ---------- Puertas ----------
pub const GATE_HEIGHT: f32 = 50.0; // alto de cada puerta
pub const GATE_SPEED: f32 = 70.0; // pixeles por segundo (hacia abajo)
pub const GATE_INTERVAL: f32 = 5.0; // segundos entre filas de puertas

// ---------- Partida ----------
pub const START_LIVES: i32 = 5;
pub const MAX_LIVES: i32 = 9; // con items de vida no pasamos de aqui
pub const START_SOLDIERS: i32 = 1;
pub const MAX_SOLDIERS: i32 = 999;

// ---------- Power-ups ----------
pub const POWERUP_DROP_CHANCE: f32 = 0.2; // 0.2 = 20% de probabilidad al destruir un enemigo
pub const POWERUP_SPEED: f32 = 110.0; // velocidad de caida
pub const POWERUP_RADIUS: f32 = 14.0; // tamano del circulo
pub const RAPID_DURATION: f32 = 6.0; // segundos que dura el disparo rapido
pub const RAPID_FIRE_FACTOR: f32 = 0.4; // el cooldown se multiplica por esto (menos = mas rapido)

// ---------- Fondo de estrellas ----------
pub const STAR_COUNT: i32 = 90; // cuantas estrellas hay
pub const STAR_MIN_SPEED: f32 = 20.0; // estrella mas lenta (lejana)
pub const STAR_MAX_SPEED: f32 = 150.0; // estrella mas rapida (cercana)

// ---------- Efectos visuales ----------
pub const PARTICLES_PER_EXPLOSION: i32 = 12; // particulas al destruir un enemigo
pub const PARTICLE_LIFE: f32 = 0.5; // segundos que dura una particula
pub const BANNER_TIME: f32 = 2.0; // segundos que se ve el aviso de nivel
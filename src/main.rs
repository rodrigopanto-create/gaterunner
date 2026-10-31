// ============================================================
// GATE RUNNER - Paso 10: musica de fondo en bucle
// ============================================================
// Objetivo de este paso:
//   1. Generar los efectos de sonido con codigo (sin archivos de audio):
//      creamos las ondas, las empaquetamos como un archivo WAV en
//      memoria y las cargamos con macroquad::audio.
//   2. Que la logica del juego solo "avise" que paso algo (un evento
//      Sfx) y que main reproduzca el sonido correspondiente.
//   3. Tecla N para silenciar / activar el sonido.
//   4. NUEVO: una melodia compuesta con codigo (acordes + arpegio +
//      bajo + ritmo) que se reproduce en bucle.
//   5. NUEVO: el volumen de la musica cambia segun el estado
//      (mas bajo en pausa, apagado en game over).
// ============================================================

// "prelude" importa todo lo comun de macroquad (colores, teclas, dibujo...)
use macroquad::prelude::*;
// Funciones de audio de macroquad
use macroquad::audio::{
    load_sound_from_bytes, play_sound, play_sound_once, set_sound_volume, PlaySoundParams, Sound,
};

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

// Audio: muestras por segundo con las que generamos los sonidos.
// 22050 es suficiente para efectos simples y mantiene los archivos pequenos.
const SAMPLE_RATE: u32 = 22050;

// Musica
const MUSIC_BPM: f32 = 130.0; // tempo: pulsos (negras) por minuto
const MUSIC_VOLUME: f32 = 0.5; // volumen maestro de la musica (0.0 a 1.0)

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

// ============================================================
// AUDIO
// ============================================================

// ---------- ENUM: eventos de sonido (Sfx = "sound effects") ----------
// La logica del juego no reproduce sonidos directamente: solo anota
// "paso esto". Asi la logica queda separada del audio.
#[derive(Clone, Copy)]
enum Sfx {
    Shoot,       // la torreta dispara
    Hit,         // una bala golpea a un enemigo
    Explosion,   // un enemigo es destruido
    GateGood,    // cruzamos una puerta buena (+ o x)
    GateBad,     // cruzamos una puerta mala (-)
    PowerUp,     // recogemos un power-up
    ShieldBreak, // el escudo absorbe un enemigo
    LifeLost,    // un enemigo cruza la linea
    LevelUp,     // subimos de nivel
    GameOver,    // perdimos
}

// ---------- ENUM: forma de onda ----------
// La forma de la onda cambia el "timbre" del sonido.
#[derive(Clone, Copy)]
enum Wave {
    Sine,   // suave y redondo
    Square, // duro, estilo videojuego retro
    Noise,  // ruido: ideal para explosiones y golpes
}

// Genera las muestras de un tono (numeros que describen la onda de sonido).
//   start_freq -> end_freq : la frecuencia (tono) cambia durante el sonido,
//                            por ejemplo de agudo a grave.
//   duration : segundos que dura.
//   volume   : de 0.0 a 1.0.
fn synth(wave: Wave, start_freq: f32, end_freq: f32, duration: f32, volume: f32) -> Vec<i16> {
    // Cuantas muestras necesitamos en total.
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    // "phase" es el punto actual dentro de la onda (en radianes).
    let mut phase: f32 = 0.0;

    for i in 0..total {
        // t avanza de 0.0 (inicio del sonido) a 1.0 (final).
        let t = i as f32 / total as f32;

        // Frecuencia actual: va de start_freq a end_freq en linea recta.
        let freq = start_freq + (end_freq - start_freq) * t;

        // Avanzamos la fase segun la frecuencia.
        phase += std::f32::consts::TAU * freq / SAMPLE_RATE as f32;

        // Valor de la onda en este instante, entre -1.0 y 1.0.
        let raw = match wave {
            Wave::Sine => phase.sin(),
            Wave::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Wave::Noise => rand::gen_range(-1.0, 1.0),
        };

        // Envolvente: el sonido empieza fuerte y se apaga hacia el final.
        // Sin esto sonaria un "clic" molesto al cortarse de golpe.
        let envelope = (1.0 - t).powi(2);

        // Convertimos a entero de 16 bits (el formato WAV), que va de -32767 a 32767.
        samples.push((raw * envelope * volume * i16::MAX as f32) as i16);
    }

    samples
}

// Empaqueta las muestras en un archivo WAV (en memoria, no en disco).
// Un WAV es una cabecera de 44 bytes + las muestras.
// Formato: PCM, 1 canal (mono), 16 bits.
fn build_wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32; // cada muestra ocupa 2 bytes
    let mut bytes: Vec<u8> = Vec::with_capacity(44 + data_len as usize);

    // to_le_bytes() convierte un numero a bytes en orden "little endian",
    // que es el que exige el formato WAV.
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes()); // tamano del archivo - 8
    bytes.extend_from_slice(b"WAVE");

    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes()); // tamano de este bloque
    bytes.extend_from_slice(&1u16.to_le_bytes()); // 1 = PCM (sin comprimir)
    bytes.extend_from_slice(&1u16.to_le_bytes()); // 1 canal (mono)
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes()); // muestras por segundo
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // bytes por segundo
    bytes.extend_from_slice(&2u16.to_le_bytes()); // bytes por muestra
    bytes.extend_from_slice(&16u16.to_le_bytes()); // bits por muestra

    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    bytes
}

// Crea un sonido listo para reproducir: sintetiza, empaqueta como WAV y carga.
async fn make_sound(wave: Wave, start_freq: f32, end_freq: f32, duration: f32, volume: f32) -> Sound {
    let samples = synth(wave, start_freq, end_freq, duration, volume);
    let wav = build_wav(&samples);
    load_sound_from_bytes(&wav)
        .await
        .expect("No se pudo crear el sonido")
}

// ---------- MUSICA ----------
// Convierte una nota MIDI en frecuencia (Hz).
// En MIDI cada numero es un semitono: 69 = La4 = 440 Hz, 60 = Do4 (do central).
// Subir 12 numeros = subir una octava = duplicar la frecuencia.
fn midi_to_freq(note: i32) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

// Genera una nota y la SUMA en un buffer de mezcla, empezando en la muestra "start".
// Sumar las ondas es como se mezclan varios instrumentos a la vez.
fn add_note(buffer: &mut [f32], start: usize, wave: Wave, freq: f32, duration: f32, volume: f32) {
    // Reutilizamos synth() de los efectos: una nota es un tono sin cambio de frecuencia.
    let samples = synth(wave, freq, freq, duration, volume);

    for (i, sample) in samples.iter().enumerate() {
        // get_mut devuelve None si el indice se sale del buffer
        // (asi una nota larga no rompe el final de la melodia).
        if let Some(slot) = buffer.get_mut(start + i) {
            *slot += *sample as f32;
        }
    }
}

// Compone la melodia completa: 4 compases que luego se repiten en bucle.
fn make_music() -> Vec<i16> {
    // Duracion de una corchea (1/8 de compas) en segundos.
    // Un pulso dura 60/BPM segundos y una corchea es medio pulso.
    let step_secs = 60.0 / MUSIC_BPM / 2.0;

    // 4 compases x 8 corcheas = 32 pasos.
    let total_steps = 32;
    let total_samples = (total_steps as f32 * step_secs * SAMPLE_RATE as f32) as usize;

    // Buffer de mezcla en decimales (f32) para poder sumar sin desbordar.
    // vec![0.0; n] crea una lista con n ceros (silencio).
    let mut buffer = vec![0.0_f32; total_samples];

    // Progresion de acordes en La menor: Am - F - C - G (muy usada en el pop y el rock).
    // Cada acorde son 3 notas MIDI: [fundamental, tercera, quinta].
    let chords: [[i32; 3]; 4] = [
        [57, 60, 64], // Am: La3, Do4, Mi4
        [53, 57, 60], // F:  Fa3, La3, Do4
        [60, 64, 67], // C:  Do4, Mi4, Sol4
        [55, 59, 62], // G:  Sol3, Si3, Re4
    ];

    // Patron del arpegio: que nota del acorde (0, 1 o 2) suena en cada corchea.
    let arp = [0usize, 1, 2, 1, 0, 1, 2, 1];

    for bar in 0..4 {
        let chord = chords[bar];

        for step in 0..8 {
            // Numero de paso en toda la cancion y muestra donde empieza.
            let global_step = bar * 8 + step;
            let start = (global_step as f32 * step_secs * SAMPLE_RATE as f32) as usize;

            // MELODIA: nota del acorde, una octava arriba (+12), onda suave.
            let lead = chord[arp[step]] + 12;
            add_note(&mut buffer, start, Wave::Sine, midi_to_freq(lead), step_secs * 1.5, 0.20);

            // BAJO: la fundamental dos octavas abajo, en los pasos 0 y 4 (tiempos 1 y 3).
            if step % 4 == 0 {
                add_note(
                    &mut buffer,
                    start,
                    Wave::Square,
                    midi_to_freq(chord[0] - 12),
                    step_secs * 3.5,
                    0.10,
                );
            }

            // RITMO: un "hi-hat" (ruido muy corto) en las corcheas impares.
            if step % 2 == 1 {
                add_note(&mut buffer, start, Wave::Noise, 0.0, 0.03, 0.05);
            }
        }
    }

    // Pasamos del buffer decimal a enteros de 16 bits, limitando los extremos.
    buffer
        .iter()
        .map(|s| s.clamp(-32767.0, 32767.0) as i16)
        .collect()
}

// Crea el sonido de la musica listo para reproducir.
async fn make_music_sound() -> Sound {
    let wav = build_wav(&make_music());
    load_sound_from_bytes(&wav)
        .await
        .expect("No se pudo crear la musica")
}

// Volumen de la musica segun el estado del juego.
fn volume_for_state(state: State) -> f32 {
    let factor = match state {
        State::Menu => 0.6,     // un poco mas suave en el menu
        State::Playing => 1.0,  // volumen completo jugando
        State::Paused => 0.25,  // bajita en pausa
        State::GameOver => 0.0, // silencio al perder
    };
    factor * MUSIC_VOLUME
}

// ---------- STRUCT SOUNDS: todos los sonidos del juego ----------
struct Sounds {
    shoot: Sound,
    hit: Sound,
    explosion: Sound,
    gate_good: Sound,
    gate_bad: Sound,
    power_up: Sound,
    shield_break: Sound,
    life_lost: Sound,
    level_up: Sound,
    game_over: Sound,
    music: Sound, // la melodia de fondo
}

impl Sounds {
    // Crea todos los sonidos al arrancar el juego.
    // Cada linea: forma de onda, tono inicial, tono final, duracion, volumen.
    async fn load() -> Sounds {
        Sounds {
            shoot: make_sound(Wave::Square, 880.0, 220.0, 0.10, 0.15).await, // "pew": agudo a grave
            hit: make_sound(Wave::Noise, 0.0, 0.0, 0.04, 0.25).await,        // golpe seco
            explosion: make_sound(Wave::Noise, 0.0, 0.0, 0.30, 0.45).await,  // ruido que se apaga
            gate_good: make_sound(Wave::Sine, 400.0, 900.0, 0.20, 0.40).await, // sube = bueno
            gate_bad: make_sound(Wave::Sine, 350.0, 100.0, 0.25, 0.40).await,  // baja = malo
            power_up: make_sound(Wave::Square, 500.0, 1200.0, 0.25, 0.18).await,
            shield_break: make_sound(Wave::Sine, 900.0, 200.0, 0.20, 0.40).await,
            life_lost: make_sound(Wave::Square, 220.0, 80.0, 0.30, 0.25).await,
            level_up: make_sound(Wave::Sine, 440.0, 1000.0, 0.40, 0.45).await,
            game_over: make_sound(Wave::Square, 400.0, 50.0, 0.90, 0.25).await,
            music: make_music_sound().await,
        }
    }

    // Reproduce el sonido que corresponde a un evento.
    fn play(&self, sfx: Sfx) {
        // match devuelve una referencia al sonido adecuado...
        let sound = match sfx {
            Sfx::Shoot => &self.shoot,
            Sfx::Hit => &self.hit,
            Sfx::Explosion => &self.explosion,
            Sfx::GateGood => &self.gate_good,
            Sfx::GateBad => &self.gate_bad,
            Sfx::PowerUp => &self.power_up,
            Sfx::ShieldBreak => &self.shield_break,
            Sfx::LifeLost => &self.life_lost,
            Sfx::LevelUp => &self.level_up,
            Sfx::GameOver => &self.game_over,
        };
        // ...y lo reproducimos una vez.
        play_sound_once(sound);
    }

    // Empieza la musica en bucle (looped: true = al terminar, vuelve a empezar).
    // La arrancamos en silencio; despues el loop principal ajusta el volumen.
    fn start_music(&self) {
        play_sound(
            &self.music,
            PlaySoundParams {
                looped: true,
                volume: 0.0,
            },
        );
    }

    // Cambia el volumen de la musica mientras sigue sonando.
    fn set_music_volume(&self, volume: f32) {
        set_sound_volume(&self.music, volume);
    }
}

// ============================================================
// JUEGO
// ============================================================

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
    powerups: Vec<PowerUp>, // power-ups cayendo
    stars: Vec<Star>,       // estrellas del fondo
    sfx: Vec<Sfx>,          // NUEVO: eventos de sonido pendientes de reproducir
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

    // NUEVO: entrega los eventos de sonido pendientes y deja la lista vacia.
    // main los usa para reproducirlos.
    fn take_sfx(&mut self) -> Vec<Sfx> {
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
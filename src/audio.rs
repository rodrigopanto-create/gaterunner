// ============================================================
// audio.rs - Efectos de sonido y reproductor
// ============================================================
// La logica del juego no reproduce sonidos directamente: solo anota
// "paso esto" con un evento Sfx. main.rs los lee y llama a Sounds::play.
// Asi la logica queda separada del audio.
// ============================================================

use macroquad::audio::{play_sound, play_sound_once, set_sound_volume, PlaySoundParams, Sound};

use crate::music::make_music_sound;
use crate::synth::{make_sound, Wave};

// ---------- ENUM: eventos de sonido (Sfx = "sound effects") ----------
#[derive(Clone, Copy)]
pub enum Sfx {
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

// ---------- STRUCT SOUNDS: todos los sonidos del juego ----------
pub struct Sounds {
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
    pub async fn load() -> Sounds {
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
    pub fn play(&self, sfx: Sfx) {
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
    pub fn start_music(&self) {
        play_sound(
            &self.music,
            PlaySoundParams {
                looped: true,
                volume: 0.0,
            },
        );
    }

    // Cambia el volumen de la musica mientras sigue sonando.
    pub fn set_music_volume(&self, volume: f32) {
        set_sound_volume(&self.music, volume);
    }
}
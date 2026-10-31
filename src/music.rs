// ============================================================
// music.rs - Melodia de fondo compuesta con codigo
// ============================================================
// Mezcla varias "voces" (melodia, bajo y ritmo) en un solo buffer
// y las convierte en un sonido que se reproduce en bucle.
// ============================================================

use macroquad::audio::{load_sound_from_bytes, Sound};

use crate::game::State;
use crate::synth::{build_wav, synth, Wave, SAMPLE_RATE};

const MUSIC_BPM: f32 = 130.0; // tempo: pulsos (negras) por minuto
const MUSIC_VOLUME: f32 = 0.5; // volumen maestro de la musica (0.0 a 1.0)

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
pub async fn make_music_sound() -> Sound {
    let wav = build_wav(&make_music());
    load_sound_from_bytes(&wav)
        .await
        .expect("No se pudo crear la musica")
}

// Volumen de la musica segun el estado del juego.
pub fn volume_for_state(state: State) -> f32 {
    let factor = match state {
        State::Menu => 0.6,     // un poco mas suave en el menu
        State::Playing => 1.0,  // volumen completo jugando
        State::Paused => 0.25,  // bajita en pausa
        State::GameOver => 0.0, // silencio al perder
    };
    factor * MUSIC_VOLUME
}
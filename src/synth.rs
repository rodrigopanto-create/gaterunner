// ============================================================
// synth.rs - Sintetizador: crea sonidos a partir de matematicas
// ============================================================
// Genera las ondas de sonido, las empaqueta como un archivo WAV
// en memoria y las carga con macroquad::audio.
// Lo usan tanto los efectos (audio.rs) como la musica (music.rs).
// ============================================================

use macroquad::audio::{load_sound_from_bytes, Sound};
use macroquad::prelude::*;

// Muestras por segundo con las que generamos los sonidos.
// 22050 es suficiente para efectos simples y mantiene los archivos pequenos.
pub const SAMPLE_RATE: u32 = 22050;

// ---------- ENUM: forma de onda ----------
// La forma de la onda cambia el "timbre" del sonido.
#[derive(Clone, Copy)]
pub enum Wave {
    Sine,   // suave y redondo
    Square, // duro, estilo videojuego retro
    Noise,  // ruido: ideal para explosiones y golpes
}

// Genera las muestras de un tono (numeros que describen la onda de sonido).
//   start_freq -> end_freq : la frecuencia (tono) cambia durante el sonido,
//                            por ejemplo de agudo a grave.
//   duration : segundos que dura.
//   volume   : de 0.0 a 1.0.
pub fn synth(wave: Wave, start_freq: f32, end_freq: f32, duration: f32, volume: f32) -> Vec<i16> {
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
pub fn build_wav(samples: &[i16]) -> Vec<u8> {
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
pub async fn make_sound(
    wave: Wave,
    start_freq: f32,
    end_freq: f32,
    duration: f32,
    volume: f32,
) -> Sound {
    let samples = synth(wave, start_freq, end_freq, duration, volume);
    let wav = build_wav(&samples);
    load_sound_from_bytes(&wav)
        .await
        .expect("No se pudo crear el sonido")
}
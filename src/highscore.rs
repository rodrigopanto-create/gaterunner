// ============================================================
// highscore.rs - Leer y guardar el record en un archivo
// ============================================================

// Archivo donde se guarda el record (en la carpeta desde donde ejecutas el juego).
// Sin "pub": solo se usa dentro de este archivo.
const HIGH_SCORE_FILE: &str = "highscore.txt";

// Lee el record guardado. Si el archivo no existe o esta danado, devuelve 0.
pub fn load_high_score() -> i32 {
    // read_to_string devuelve un "Result" (puede fallar). Con .ok() lo
    // convertimos en Option, con and_then/parse intentamos convertir el
    // texto a numero, y con unwrap_or(0) usamos 0 si algo fallo.
    std::fs::read_to_string(HIGH_SCORE_FILE)
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

// Guarda el record como texto en el archivo.
pub fn save_high_score(score: i32) {
    // "let _ =" ignora el resultado: si no se puede guardar, el juego sigue.
    let _ = std::fs::write(HIGH_SCORE_FILE, score.to_string());
}
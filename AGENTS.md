# 🚀 AGENTS.MD - Gate Runner (Rust + Macroquad)

## 📌 Visión del Proyecto
Desarrollar un videojuego educativo en **Rust** usando **Macroquad** que combine:
1. **Mecánica de Multiplicación / Corredor:** Movimiento horizontal para atravesar puertas flotantes con modificadores (`+N`, `-N`, `xN`, `/N`).
2. **Shoot 'em Up:** Disparo automático hacia arriba contra hordas o bloques enemigos que descienden.
3. **Tower Defense / Línea de Defensa:** Prevenir que los enemigos crucen la parte inferior de la pantalla o destruyan tu base.

---

## 🛠️ Stack Tecnológico & Dependencias
- **Lenguaje:** Rust (Edición 2021)
- **Motor / Gráficos:** `macroquad` (0.4+)
- **Enfoque de diseño:** Código limpio, iterativo, bien comentado y sin librerías externas complejas en las primeras etapas (usando primitivas geométricas 2D: rectángulos, círculos y texto).

---

## 🎯 Principios de Código & Guía de Estilo
- **Simplicidad ante todo:** Priorizar claridad educativa sobre micro-optimizaciones prematuras.
- **Comentarios explicativos:** Explicar el *porqué* de cada estructura de Rust (`Vec`, `struct`, `match`, ownership en bucles).
- **Modularidad progresiva:** Iniciar en un solo archivo `main.rs` para captar la idea global, y refactorizar a módulos (`player.rs`, `gates.rs`, `bullets.rs`, `enemies.rs`) conforme crezca el proyecto.
- **Delta Time (`get_frame_time()`):** Todo el movimiento debe estar basado en tiempo delta para asegurar la misma velocidad en cualquier framerate.

---

## 🗺️ Hoja de Ruta / Fases de Desarrollo

### 🟢 Fase 1: Fundamentos y Ventana Principal
- [ ] Configuración inicial de Cargo y `Cargo.toml`.
- [ ] Inicialización de la ventana de Macroquad (`Conf`).
- [ ] Bucle principal (`loop`) y fondo de pantalla.

### 🟡 Fase 2: El Jugador y Movimiento
- [ ] Representación visual del jugador/equipo (círculo o conjunto de partículas/soldados).
- [ ] Control horizontal (Teclas Izquierda/Derecha o A/D y soporte opcional de Mouse/Touch).
- [ ] Definición de límites de la pantalla (pantalla como carril).

### 🔵 Fase 3: Sistema de Puertas (Gates)
- [ ] Estructura `Gate` con posición, tamaño, tipo de operación (`Add`, `Subtract`, `Multiply`) y valor.
- [ ] Generación y descenso continuo de puertas.
- [ ] Detección de colisión (AABB: *Axis-Aligned Bounding Box*) entre el jugador y las puertas.
- [ ] Aplicación del modificador al contador/ejército del jugador.

### 🔴 Fase 4: Disparo y Sistema de Enemigos (Shmup + Tower Defense)
- [ ] Disparo automático del jugador (proyectiles que viajan hacia arriba).
- [ ] Generación de enemigos o bloques destructibles que bajan con puntos de vida (HP).
- [ ] Colisiones Proyectil vs Enemigo.
- [ ] Línea de defensa inferior: Pérdida de vidas / Game Over si un enemigo cruza la línea.

### 🟣 Fase 5: UI, Pulido y Game Loop Completo
- [ ] Marcadores en pantalla: Contador de tropa/potencia, HP del jugador/base, Score.
- [ ] Pantallas de estado: `Menu`, `Playing`, `GameOver`.
- [ ] Efectos visuales simples (colores por tipo de puerta, partículas básicas).

---

## 📂 Estructura Sugerida del Proyecto (Fase Avanzada)

```text
gaterunner/
├── Cargo.toml
├── AGENTS.MD
└── src/
    ├── main.rs         # Bucle principal y configuración de estados
    ├── player.rs       # Entidad del jugador, recuento y disparo
    ├── gate.rs         # Lógica de puertas y operaciones matemáticas
    ├── enemy.rs        # Enemigos, hordas y bloques defensivos
    └── bullet.rs       # Gestión de proyectiles
# AI-2048 — Resumen de cambios e instrucciones

Este repositorio contiene una implementación de un agente (Expectimax) y una UI para el juego 2048. Se han añadido funcionalidades de estimación bayesiana, visualización de probabilidades y estadísticas en tiempo de ejecución.

---

## Resumen de los cambios realizados

1. Nuevo módulo Bayes
   - `src/bayes.rs`: implementación de un estimador bayesiano (Beta prior) que mantiene una estimación posterior de la probabilidad de que el siguiente tile sea `4` (en lugar de `2`).
   - API pública principal:
     - `bayes::enable()` / `bayes::disable()` — activar/desactivar el registro de estadísticas.
     - `bayes::reset_stats_for_new_game()` — reiniciar las estadísticas al comenzar una nueva partida (llamado desde la UI cuando se inicia un juego).
     - `bayes::record_observed_two()` / `bayes::record_observed_four()` — funciones que se invocan al generar un tile para actualizar la posterior.
     - `bayes::record_p_move_prob(f32)` — registra una probabilidad por movimiento (historial para sparkline).
     - `bayes::get_stats()` — devuelve una vista de las estadísticas para la UI.
     - `bayes::print_summary()` — imprime un resumen en consola al finalizar la partida.

2. Integración en la lógica del juego
   - `src/board.rs`:
     - `Board::add_random()` ahora filtra las observaciones y llama a `bayes::record_observed_two()` o `bayes::record_observed_four()` cuando corresponde.
     - `PlayableBoard::draw()` se amplió para mostrar en la UI:
       - la probabilidad posterior actual P(next tile=4),
       - overlays en cada celda vacía con la probabilidad por celda de recibir un `4` (posterior / #vacías),
       - una barra con el recuento de `4` observados,
       - un pequeño sparkline con el historial de probabilidades registradas por movimiento.
   - `src/search.rs`:
     - tras elegir la acción con Expectimax se registra una entrada en el historial `bayes::record_p_move_prob(...)` (por defecto graba la media posterior actual; se puede cambiar para registrar la probabilidad esperada exacta por estado).

3. UI / ejecución
   - `src/main.rs`:
     - al iniciar una partida en modo Agente (`A`) o Humano (`P`) se llama a `bayes::reset_stats_for_new_game()` y `bayes::enable()` para empezar a recolectar estadísticas.
     - al finalizar la partida se imprime un resumen de las estadísticas Bayes en la consola (`bayes::print_summary()`).

4. Cambios en `bench`
   - `src/bench.rs` ahora incluye `mod bayes;` para evitar problemas de compilación al construir los dos binarios.
   - `bench` mantiene su comportamiento de jugar muchas partidas en paralelo; por defecto `bayes` está en modo desactivado en el contexto de `bench` (si quieres habilitar recolección por cada worker hay que agregar sincronización/aggregación adicional).

5. Dependencias añadidas
   - `once_cell = "1.17"` se añadió en `Cargo.toml` para almacenar el estado global de estadísticas de manera segura.

---

## Qué verás en la interfaz (Macroquad)
- Texto en la UI con: "P(next tile is 4) = XX.XX%" (estimación posterior actual).
- Overlay azul translúcido en celdas vacías con el porcentaje por celda (pequeño texto dentro de la celda).
- Barra que indica el total de `4` observados en la partida.
- Sparkline (línea) con el historial de probabilidades por movimiento (hasta las últimas ~200 entradas).

Al terminar la partida (GAME OVER) se imprime por consola un resumen con la media posterior p(4), recuentos de `2` y `4` observados, y estadísticas del historial si existen.

---

## Cómo compilar y ejecutar

Compilar el proyecto:

```bash
cargo build
```

Ejecutar la UI (binario principal):

```bash
cargo run --bin main
```

Al iniciar te pedirá modo:
- `A` -> Agent Mode (Expectimax)
- `P` -> Human Mode (teclas W/A/S/D o flechas)

Ejecutar la versión de bench (sin UI):

```bash
cargo run --bin bench -- -n 8 -t 600
```

Ajusta `-n` (número de juegos) y `-t` (timeout en segundos) según desees.

---

## Notas técnicas y consideraciones
- El modelo Bayesiano es un Beta simple con prior (a=1, b=9) para representar la creencia inicial p(4)=0.1.
- `Board::add_random()` mantiene la lógica original de generación (prob. 0.9 para `2` y 0.1 para `4`), pero ahora reporta las observaciones al módulo Bayes.
- La probabilidad por celda que se muestra en la UI es `posterior_mean / num_empty` (sencilla y fácil de interpretar). Si quieres que el valor mostrado sea la probabilidad teórica basada en el árbol de sucesores para ese estado, puedo cambiar la grabación en `search` para computarla a partir de `RandableBoard::successors()` en vez de usar la posterior global.

---


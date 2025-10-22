use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Estructura interna para estadísticas y parámetros Beta.
#[derive(Clone, Debug)]
struct InnerStats {
    // Beta prior parameters: a for "success" (tile==4), b for "failure" (tile==2)
    a: u32,
    b: u32,
    total_fours: u32,
    total_twos: u32,
    p_move_probs: Vec<f32>, // historial de probabilidades esperadas por turno
    enabled: bool,
}

impl Default for InnerStats {
    fn default() -> Self {
        // Prior: expectation 0.1 => a/(a+b) = 0.1 -> choose a=1, b=9
        InnerStats { a: 1, b: 9, total_fours: 0, total_twos: 0, p_move_probs: Vec::new(), enabled: false }
    }
}

static STATS: Lazy<Mutex<InnerStats>> = Lazy::new(|| Mutex::new(InnerStats::default()));

/// Vista publica de las estadísticas para uso en UI.
#[derive(Clone, Debug)]
pub struct StatsView {
    pub posterior_mean: f32,
    pub total_fours: u32,
    pub total_twos: u32,
    pub p_move_probs: Vec<f32>,
}

pub fn enable() {
    let mut s = STATS.lock().unwrap();
    s.enabled = true;
}

pub fn disable() {
    let mut s = STATS.lock().unwrap();
    s.enabled = false;
}

pub fn reset_stats_for_new_game() {
    let mut s = STATS.lock().unwrap();
    *s = InnerStats::default();
    s.enabled = true; // keep enabled after reset when called from UI
}

pub fn record_observed_two() {
    let mut s = STATS.lock().unwrap();
    if !s.enabled { return; }
    s.b += 1;
    s.total_twos += 1;
}

pub fn record_observed_four() {
    let mut s = STATS.lock().unwrap();
    if !s.enabled { return; }
    s.a += 1;
    s.total_fours += 1;
}

/// Registra la probabilidad esperada (según el árbol de búsqueda) de que el siguiente tile sea 4 en este movimiento.
pub fn record_p_move_prob(p: f32) {
    let mut s = STATS.lock().unwrap();
    if !s.enabled { return; }
    // mantener sólo las últimas 200 entradas para evitar crecer indefinidamente
    s.p_move_probs.push(p.clamp(0.0, 1.0));
    if s.p_move_probs.len() > 200 {
        let excess = s.p_move_probs.len() - 200;
        s.p_move_probs.drain(0..excess);
    }
}

pub fn get_stats() -> StatsView {
    let s = STATS.lock().unwrap();
    let posterior_mean = (s.a as f32) / ((s.a + s.b) as f32);
    StatsView { posterior_mean, total_fours: s.total_fours, total_twos: s.total_twos, p_move_probs: s.p_move_probs.clone() }
}

pub fn print_summary() {
    let s = STATS.lock().unwrap();
    if !s.enabled { return; }
    println!("Bayes posterior mean p(4) = {:.4}", (s.a as f32) / ((s.a + s.b) as f32));
    println!("Total observed 4s: {}", s.total_fours);
    println!("Total observed 2s: {}", s.total_twos);
    if !s.p_move_probs.is_empty() {
        let avg: f32 = s.p_move_probs.iter().sum::<f32>() / (s.p_move_probs.len() as f32);
        println!("Average recorded p_move (last {}): {:.4}", s.p_move_probs.len(), avg);
    }
}

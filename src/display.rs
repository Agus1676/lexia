//! display.rs — Formateo visual en la terminal (colores, spinner, layouts)
//!
//! Lexia: Lector inteligente de documentos con IA
//! Autor: Agustín Pollán

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// ── Constantes ─────────────────────────────────────────────────────────────

const SEPARATOR: &str = "──────────────────────────────────────────";

// ── Funciones públicas ─────────────────────────────────────────────────────

/// Imprime el banner de bienvenida de Lexia.
pub fn print_banner() {
    println!();
    println!("{}", "╔════════════════════════════════════════╗".bright_purple());
    println!("{}", "║      📖  Lexia  ·  by Agustín Pollán   ║".bright_purple());
    println!("{}", "║   Lector inteligente de documentos     ║".bright_purple());
    println!("{}", "╚════════════════════════════════════════╝".bright_purple());
    println!();
}

/// Muestra información básica del archivo siendo procesado.
pub fn print_file_info(filename: &str, char_count: usize) {
    println!("{} {}", "📄 Archivo:".bright_cyan().bold(), filename.white());
    println!(
        "{} {} caracteres extraídos",
        "📊 Tamaño:".bright_cyan().bold(),
        char_count.to_string().yellow()
    );
    println!();
}

/// Crea y retorna un spinner animado con el mensaje dado.
pub fn start_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Muestra el resultado del análisis con título y separadores.
pub fn print_result(title: &str, content: &str) {
    println!("{}", SEPARATOR.bright_purple());
    println!("{}", title.bright_yellow().bold());
    println!("{}", SEPARATOR.bright_purple());
    println!();

    for line in content.lines() {
        if line.trim().is_empty() {
            println!();
        } else if line.starts_with('•') {
            // Bullet points: destacar símbolo
            let body = line.trim_start_matches('•').trim();
            println!("  {} {}", "•".bright_cyan().bold(), body.white());
        } else {
            println!("  {}", line.white());
        }
    }

    println!();
    println!("{}", SEPARATOR.bright_purple());
    println!();
}

/// Muestra un mensaje de error en stderr.
pub fn print_error(message: &str) {
    eprintln!();
    eprintln!("{} {}", "❌ Error:".bright_red().bold(), message.red());
    eprintln!();
}

/// Muestra un mensaje de éxito.
pub fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message.bright_green());
}

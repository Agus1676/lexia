//! main.rs — Punto de entrada de Lexia
//!
//! Lexia: Lector inteligente de documentos con IA
//! Autor: Agustín Pollán

mod display;
mod extractor;
mod gemini;
mod server;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

// ── CLI ────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name    = "lexia",
    about   = "📖 Lexia — Lector inteligente de documentos con IA",
    version = "0.1.0",
    author  = "Agustín Pollán"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Genera un resumen conciso o detallado del documento
    Resumen {
        /// Ruta al archivo (.pdf, .txt, .md)
        archivo: PathBuf,

        /// Idioma del resumen (ej: español, inglés, portugués)
        #[arg(long, default_value = "español")]
        idioma: String,

        /// Genera un resumen más largo y detallado
        #[arg(long)]
        largo: bool,
    },

    /// Extrae y lista los puntos más importantes del documento
    #[command(name = "puntos-clave")]
    PuntosClave {
        /// Ruta al archivo (.pdf, .txt, .md)
        archivo: PathBuf,

        /// Idioma de los puntos clave (ej: español, inglés)
        #[arg(long, default_value = "español")]
        idioma: String,
    },

    /// Inicia la interfaz web de Lexia
    Serve {
        /// Puerto donde correr el servidor web
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}

// ── Entry point ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Command::Serve { port } => {
            validate_api_key();
            // Si Render nos pasa la variable PORT, la priorizamos
            let final_port = std::env::var("PORT")
                .ok()
                .and_then(|val| val.parse::<u16>().ok())
                .unwrap_or(port);
            server::run(final_port).await;
        }

        Command::Resumen { archivo, idioma, largo } => {
            display::print_banner();
            validate_api_key();
            let api_key = std::env::var("GEMINI_API_KEY").unwrap();
            run_analysis(&api_key, &archivo, &idioma, "resumen", largo).await;
        }

        Command::PuntosClave { archivo, idioma } => {
            display::print_banner();
            validate_api_key();
            let api_key = std::env::var("GEMINI_API_KEY").unwrap();
            run_analysis(&api_key, &archivo, &idioma, "puntos-clave", false).await;
        }
    }
}

// ── Lógica compartida de análisis (modo CLI) ───────────────────────────────

async fn run_analysis(api_key: &str, archivo: &PathBuf, idioma: &str, modo: &str, largo: bool) {
    // Verificar que el archivo existe
    if !archivo.exists() {
        display::print_error(&format!("No se encontró el archivo: {}", archivo.display()));
        std::process::exit(1);
    }

    let filename = archivo
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("desconocido");

    // Extraer texto
    let spinner = display::start_spinner("Extrayendo texto del documento...");
    let text = match extractor::extract_text(archivo) {
        Ok(t)  => { spinner.finish_and_clear(); t }
        Err(e) => { spinner.finish_and_clear(); display::print_error(&e.to_string()); std::process::exit(1); }
    };

    display::print_file_info(filename, text.len());

    // Construir prompt y consultar a Gemini
    let prompt = match modo {
        "puntos-clave" => gemini::keypoints_prompt(&text, idioma),
        _              => gemini::summary_prompt(&text, idioma, largo),
    };

    let spinner = display::start_spinner("Analizando con Gemini...");
    let result = match gemini::ask(api_key, &prompt).await {
        Ok(r)  => { spinner.finish_and_clear(); r }
        Err(e) => { spinner.finish_and_clear(); display::print_error(&e); std::process::exit(1); }
    };

    // Mostrar resultado
    let title = match modo {
        "puntos-clave" => format!("📌 Puntos Clave — {}", filename),
        _              => format!("📝 Resumen — {}", filename),
    };

    display::print_result(&title, &result);
    display::print_success("Documento procesado exitosamente.");
}

// ── Utilidades ─────────────────────────────────────────────────────────────

fn validate_api_key() {
    if std::env::var("GEMINI_API_KEY").is_err() {
        eprintln!("❌ GEMINI_API_KEY no encontrada. Creá un archivo .env con tu clave.");
        std::process::exit(1);
    }
}

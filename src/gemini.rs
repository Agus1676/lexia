//! gemini.rs — Cliente HTTP para la API de Google Gemini
//!
//! Lexia: Lector inteligente de documentos con IA
//! Autor: Agustín Pollán

use reqwest::Client;
use serde::{Deserialize, Serialize};

const API_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-lite-latest:generateContent";

const MAX_TEXT_LENGTH: usize = 30_000;

// ── Tipos de la petición ───────────────────────────────────────────────────

#[derive(Serialize)]
struct Request {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

// ── Tipos de la respuesta ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct Response {
    candidates: Option<Vec<Candidate>>,
    error:      Option<ApiError>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Deserialize)]
struct PartResponse {
    text: String,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

// ── Función principal ──────────────────────────────────────────────────────

/// Envía un prompt a Gemini y devuelve el texto generado.
pub async fn ask(api_key: &str, prompt: &str) -> Result<String, String> {
    let client = Client::new();
    let url    = format!("{}?key={}", API_URL, api_key);

    let body = Request {
        contents: vec![Content {
            parts: vec![Part { text: prompt.to_string() }],
        }],
    };

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {}", e))?;

    let parsed: Response = response
        .json()
        .await
        .map_err(|e| format!("Error al leer respuesta: {}", e))?;

    if let Some(err) = parsed.error {
        return Err(format!("Error de API: {}", err.message));
    }

    parsed
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| "La API no devolvió contenido.".to_string())
}

// ── Prompts ────────────────────────────────────────────────────────────────

/// Prompt para generar un resumen del documento.
pub fn summary_prompt(text: &str, idioma: &str, largo: bool) -> String {
    let longitud = if largo { "detallado (4-6 párrafos)" } else { "conciso (2-3 párrafos)" };
    let truncado = &text[..text.len().min(MAX_TEXT_LENGTH)];

    format!(
        "Eres un asistente experto en análisis de documentos. \
        Generá un resumen {} del siguiente texto, en {}. \
        Capturá las ideas principales y conclusiones más relevantes. \
        Ir directo al contenido, sin frases introductorias.\n\n\
        === DOCUMENTO ===\n{}\n================",
        longitud, idioma, truncado
    )
}

/// Prompt para extraer los puntos clave del documento.
pub fn keypoints_prompt(text: &str, idioma: &str) -> String {
    let truncado = &text[..text.len().min(MAX_TEXT_LENGTH)];

    format!(
        "Eres un asistente experto en análisis de documentos. \
        Extraé los puntos más importantes del siguiente texto en {}. \
        Presentalos como lista ordenada por importancia (máximo 10 puntos). \
        Formato exacto por punto: '• [punto clave]'\n\n\
        === DOCUMENTO ===\n{}\n================",
        idioma, truncado
    )
}

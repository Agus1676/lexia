//! server.rs — Servidor web Axum de Lexia
//!
//! Expone un endpoint POST /api/process que recibe un archivo
//! (multipart/form-data), extrae su texto y lo procesa con Gemini.
//!
//! Lexia: Lector inteligente de documentos con IA
//! Autor: Agustín Pollán

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::gemini;

// ── Tipos ──────────────────────────────────────────────────────────────────

/// Respuesta estándar de la API.
#[derive(Serialize)]
struct ApiResponse {
    success:    bool,
    result:     Option<String>,
    error:      Option<String>,
    filename:   Option<String>,
    char_count: Option<usize>,
}

impl ApiResponse {
    fn ok(result: String, filename: String, char_count: usize) -> Self {
        Self { success: true, result: Some(result), error: None, filename: Some(filename), char_count: Some(char_count) }
    }

    fn err(error: String, filename: Option<String>) -> Self {
        Self { success: false, result: None, error: Some(error), filename, char_count: None }
    }
}

/// Parámetros extraídos del formulario multipart.
struct FormData {
    file_bytes: Vec<u8>,
    filename:   String,
    mode:       String,
    idioma:     String,
    largo:      bool,
}

// ── Handlers ───────────────────────────────────────────────────────────────

async fn health() -> &'static str {
    "Lexia OK 📖"
}

async fn process(mut multipart: Multipart) -> impl IntoResponse {
    // Validar API key
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(k)  => k,
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("GEMINI_API_KEY no configurada.".into(), None)),
        ),
    };

    // Parsear formulario multipart
    let form = match parse_multipart(&mut multipart).await {
        Ok(f)  => f,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e, None))),
    };

    // Extraer texto del archivo según su extensión
    let ext = form.filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let text = match extract_text(&form.file_bytes, &ext) {
        Ok(t)  => t,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e, Some(form.filename)))),
    };

    let char_count = text.len();

    // Construir prompt y llamar a Gemini
    let prompt = match form.mode.as_str() {
        "puntos-clave" => gemini::keypoints_prompt(&text, &form.idioma),
        _              => gemini::summary_prompt(&text, &form.idioma, form.largo),
    };

    match gemini::ask(&api_key, &prompt).await {
        Ok(result) => (StatusCode::OK, Json(ApiResponse::ok(result, form.filename, char_count))),
        Err(e)     => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e, Some(form.filename)))),
    }
}

// ── Helpers internos ───────────────────────────────────────────────────────

/// Lee todos los campos del formulario multipart y los agrupa en `FormData`.
async fn parse_multipart(multipart: &mut Multipart) -> Result<FormData, String> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = String::from("documento");
    let mut mode     = String::from("resumen");
    let mut idioma   = String::from("español");
    let mut largo    = false;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "file" => {
                filename   = field.file_name().unwrap_or("documento").to_string();
                file_bytes = Some(field.bytes().await.map_err(|e| e.to_string())?.to_vec());
            }
            "mode"   => { if let Ok(v) = field.text().await { mode   = v; } }
            "idioma" => { if let Ok(v) = field.text().await { idioma = v; } }
            "largo"  => { if let Ok(v) = field.text().await { largo  = v == "true"; } }
            _        => {}
        }
    }

    Ok(FormData {
        file_bytes: file_bytes.ok_or("No se recibió ningún archivo.")?,
        filename,
        mode,
        idioma,
        largo,
    })
}

/// Extrae el texto del buffer de bytes según la extensión del archivo.
fn extract_text(bytes: &[u8], ext: &str) -> Result<String, String> {
    match ext {
        "pdf" => {
            let text = pdf_extract::extract_text_from_mem(bytes)
                .map_err(|e| format!("Error al procesar el PDF: {}", e))?;
            if text.trim().is_empty() {
                return Err("No se pudo extraer texto del PDF (puede ser una imagen escaneada).".into());
            }
            Ok(text)
        }
        "txt" | "md" => {
            String::from_utf8(bytes.to_vec())
                .map_err(|_| "El archivo de texto no tiene codificación UTF-8 válida.".into())
        }
        other => Err(format!("Formato '.{}' no soportado. Usá PDF, TXT o MD.", other)),
    }
}

// ── Iniciar servidor ───────────────────────────────────────────────────────

/// Inicia el servidor Axum en el puerto indicado.
pub async fn run(port: u16) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/process", post(process))
        .nest_service("/", ServeDir::new("static"))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("📖 Lexia corriendo en http://localhost:{}", port);
    println!("   Desarrollado por Agustín Pollán\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

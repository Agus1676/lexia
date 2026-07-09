//! extractor.rs — Extracción de texto desde archivos PDF y TXT
//!
//! Lexia: Lector inteligente de documentos con IA
//! Autor: Agustín Pollán

use std::{fmt, fs, io, path::Path};

// ── Error ──────────────────────────────────────────────────────────────────

pub enum Error {
    Io(io::Error),
    Pdf(String),
    UnsupportedFormat(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e)                  => write!(f, "Error al leer el archivo: {}", e),
            Error::Pdf(e)                 => write!(f, "Error al procesar el PDF: {}", e),
            Error::UnsupportedFormat(ext) => write!(f, "Formato no soportado: '.{}'. Usá .pdf o .txt", ext),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

// ── Función pública ────────────────────────────────────────────────────────

/// Detecta la extensión del archivo y extrae su contenido de texto.
pub fn extract_text(path: &Path) -> Result<String, Error> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" | "md" => read_txt(path),
        "pdf"        => read_pdf(path),
        other        => Err(Error::UnsupportedFormat(other.to_string())),
    }
}

// ── Funciones internas ─────────────────────────────────────────────────────

fn read_txt(path: &Path) -> Result<String, Error> {
    let content = fs::read_to_string(path)?;

    if content.trim().is_empty() {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "El archivo de texto está vacío.",
        )));
    }

    Ok(content)
}

fn read_pdf(path: &Path) -> Result<String, Error> {
    let bytes = fs::read(path)?;
    let text  = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| Error::Pdf(e.to_string()))?;

    if text.trim().is_empty() {
        return Err(Error::Pdf(
            "No se pudo extraer texto del PDF (puede ser una imagen escaneada).".to_string(),
        ));
    }

    Ok(text)
}

# LEXIA — Lector inteligente de documentos con IA

Servidor web y cliente de línea de comandos en Rust para procesar, resumir y extraer puntos clave de archivos PDF, TXT y MD utilizando la API de Google Gemini.

El proyecto incluye un frontend minimalista en HTML y CSS vainilla con un canvas interactivo en 3D que implementa un universo de partículas que reacciona a la posición del cursor de forma elástica, simulando gravedad cero y repulsión física.

## Características

*   **Extracción en memoria:** Parseo de texto directo de archivos PDF (mediante `pdf-extract`) y archivos de texto plano sin almacenar datos temporales en disco.
*   **Integración con Gemini:** Uso del modelo `gemini-2.0-flash-lite` optimizado en cuota mediante peticiones estructuradas HTTP asíncronas.
*   **Fondo interactivo 3D:** Renderizado mediante canvas 2D con proyección de perspectiva tridimensional y física de repulsión elástica basada en la posición del mouse.
*   **Exportación a PDF:** Generación de reportes formateados del lado del cliente utilizando `jsPDF` con estilos coherentes al sistema de diseño.
*   **Monolítico y Ligero:** Servidor HTTP asíncrono construido sobre `Axum` y `Tokio` que sirve tanto la API JSON como los archivos estáticos.

## Estructura del Proyecto

*   `/src`: Código fuente del servidor y cliente CLI en Rust.
    *   `main.rs`: Punto de entrada de la aplicación y gestión de comandos CLI.
    *   `server.rs`: Configuración del servidor Axum y endpoints HTTP.
    *   `gemini.rs`: Cliente HTTP para la API de Google Gemini.
    *   `extractor.rs`: Lector y decodificador de buffers de texto (PDF, TXT, MD).
*   `/static`: Frontend de la aplicación.
    *   `index.html`: Estructura semántica de la interfaz.
    *   `design.css`: Sistema de diseño minimalista claro con acentos en rojo carmín.
    *   `app.js`: Lógica de carga de archivos, llamadas a la API y el motor de partículas 3D.

## Requisitos Previos

*   [Rust](https://www.rust-lang.org/) (Edición 2021 o superior)
*   Compilador C / Linker compatible en el sistema (por ejemplo, GCC / MSVC)
*   Una API Key de [Google AI Studio](https://aistudio.google.com/)

## Configuración y Despliegue Local

1.  **Clonar el repositorio:**
    ```bash
    git clone https://github.com/tu-usuario/lexia.git
    cd lexia
    ```

2.  **Configurar variables de entorno:**
    Crea un archivo `.env` en la raíz del proyecto y agrega tu clave de la API de Gemini:
    ```env
    GEMINI_API_KEY=tu_api_key_aqui
    ```

3.  **Compilar la aplicación:**
    ```bash
    cargo build --release
    ```

4.  **Iniciar el servidor web:**
    ```bash
    ./target/release/lexia serve --port 8080
    ```
    La aplicación estará disponible en `http://localhost:8080`.

5.  **Uso de la interfaz de comandos (CLI):**
    También puedes procesar archivos directamente desde la terminal:
    ```bash
    # Obtener resumen
    ./target/release/lexia resumen /ruta/al/documento.pdf
    
    # Extraer puntos clave
    ./target/release/lexia puntos-clave /ruta/al/documento.txt
    ```

---

Desarrollado por [Agustín Pollán](https://github.com/tu-usuario).

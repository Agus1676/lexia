// app.js — Lógica de navegación e interacción del frontend de Lexia

// ── Estado Local ────────────────────────────────────────────────────────────
const state = {
    file:         null,
    mode:         'resumen',
    idioma:       'español',
    largo:        false,
    lastResult:   '',
    lastFilename: '',
};

// ── Helper de Selección DOM ─────────────────────────────────────────────────
const $ = id => document.getElementById(id);

// ── Vistas del Flujo ────────────────────────────────────────────────────────
const views = {
    main:    $('view-main'),
    loading: $('view-loading'),
    result:  $('view-result'),
    error:   $('view-error'),
};

function showView(name) {
    Object.entries(views).forEach(([key, el]) => {
        el.classList.toggle('hidden', key !== name);
    });
}

// ── Referencias de Componentes ──────────────────────────────────────────────
const dropzone       = $('dropzone');
const fileInput      = $('file-input');
const dzEmpty        = $('dropzone-empty');
const dzSelected     = $('dropzone-selected');
const fileName       = $('file-name');
const fileSize       = $('file-size');
const fileThumb      = $('file-thumb');
const fileRemove     = $('file-remove');
const btnProcess     = $('btn-process');
const modeToggle     = $('mode-toggle');
const largoGroup     = $('largo-group');
const largoToggle    = $('largo-toggle');
const idiomaSelect   = $('idioma-select');
const resultContent  = $('result-content');
const resultLabel    = $('result-label');
const resultFilename = $('result-filename');
const statChars      = $('stat-chars');
const metaMode       = $('meta-mode');
const errorMessage   = $('error-message');
const btnCopy        = $('btn-copy');
const btnDownload    = $('btn-download');
const btnNew         = $('btn-new');
const btnRetry       = $('btn-retry');

// ── Configuración / Inputs ──────────────────────────────────────────────────
modeToggle.querySelectorAll('.seg-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        modeToggle.querySelectorAll('.seg-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        state.mode = btn.dataset.value;
        largoGroup.style.display = state.mode === 'resumen' ? '' : 'none';
    });
});

largoToggle.querySelectorAll('.seg-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        largoToggle.querySelectorAll('.seg-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        state.largo = btn.dataset.value === 'true';
    });
});

idiomaSelect.addEventListener('change', () => {
    state.idioma = idiomaSelect.value;
});

// ── Drag and Drop / Subida de Archivo ───────────────────────────────────────
dropzone.addEventListener('click', e => {
    if (fileRemove && fileRemove.contains(e.target)) return;
    fileInput.click();
});

fileInput.addEventListener('change', () => {
    if (fileInput.files[0]) setFile(fileInput.files[0]);
});

dropzone.addEventListener('dragover', e => {
    e.preventDefault();
    dropzone.classList.add('drag-over');
});

dropzone.addEventListener('dragleave', () => {
    dropzone.classList.remove('drag-over');
});

dropzone.addEventListener('drop', e => {
    e.preventDefault();
    dropzone.classList.remove('drag-over');
    if (e.dataTransfer.files[0]) setFile(e.dataTransfer.files[0]);
});

function setFile(file) {
    const ext = file.name.split('.').pop().toLowerCase();
    const valid = ['pdf', 'txt', 'md'];

    if (!valid.includes(ext)) {
        showError(`El archivo no es válido. Formato ".${ext}" no soportado. Usá PDF, TXT o MD.`);
        return;
    }

    state.file = file;
    fileName.textContent = file.name;
    fileSize.textContent = formatBytes(file.size);
    fileThumb.textContent = ext.toUpperCase();
    
    dzEmpty.classList.add('hidden');
    dzSelected.classList.remove('hidden');
    btnProcess.disabled = false;
}

if (fileRemove) {
    fileRemove.addEventListener('click', e => {
        e.stopPropagation();
        clearFile();
    });
}

function clearFile() {
    state.file = null;
    fileInput.value = '';
    dzEmpty.classList.remove('hidden');
    dzSelected.classList.add('hidden');
    btnProcess.disabled = true;
}

// ── Petición al Servidor ────────────────────────────────────────────────────
async function processDocument() {
    if (!state.file) return;

    showView('loading');

    const formData = new FormData();
    formData.append('file',   state.file);
    formData.append('mode',   state.mode);
    formData.append('idioma', state.idioma);
    formData.append('largo',  String(state.largo));

    try {
        const res = await fetch('/api/process', { method: 'POST', body: formData });
        const data = await res.json();

        if (data.success) {
            state.lastResult   = data.result;
            state.lastFilename = data.filename || state.file.name;
            renderResult(data);
            showView('result');
        } else {
            showError(data.error || 'Error al procesar el archivo.');
        }
    } catch (err) {
        showError(`Error al conectar con la API: ${err.message}`);
    }
}

btnProcess.addEventListener('click', processDocument);

// ── Renderizado del Resultado ───────────────────────────────────────────────
function renderResult(data) {
    const isPuntos = state.mode === 'puntos-clave';
    const label    = isPuntos ? 'Puntos clave' : 'Resumen';

    resultLabel.textContent    = label;
    resultFilename.textContent = data.filename || state.file.name;
    metaMode.textContent       = label;
    statChars.textContent      = `${(data.char_count || 0).toLocaleString()} chars`;

    if (isPuntos) {
        const lines = data.result.split('\n');
        const bullets = lines.filter(l => l.trim() && /^[•\-\*]/.test(l.trim()));

        if (bullets.length > 0) {
            resultContent.innerHTML = bullets.map((line, i) => {
                const text = line.replace(/^[•\-\*]\s*/, '').trim();
                return `<div class="kp-item">
                    <span class="kp-num">${String(i + 1).padStart(2, '0')}</span>
                    <span>${escapeHtml(text)}</span>
                </div>`;
            }).join('');
        } else {
            resultContent.innerHTML = renderParagraphs(data.result);
        }
    } else {
        resultContent.innerHTML = renderParagraphs(data.result);
    }
}

function renderParagraphs(text) {
    return text
        .split('\n\n')
        .filter(p => p.trim())
        .map(p => `<p>${escapeHtml(p.trim())}</p>`)
        .join('');
}

// ── Error ───────────────────────────────────────────────────────────────────
function showError(msg) {
    let cleanMsg = msg;
    
    // Simplificar errores comunes de API de forma elegante
    if (msg.includes("quota") || msg.includes("limit") || msg.includes("exceeded")) {
        cleanMsg = "Se ha superado el límite de peticiones gratuitas de la API de Gemini. Por favor, esperá unos segundos e intentá de nuevo.";
    } else if (msg.includes("API key") || msg.includes("key not found")) {
        cleanMsg = "La clave de la API de Gemini no es válida o no está configurada en el archivo .env.";
    } else if (msg.includes("Failed to fetch") || msg.includes("conectar")) {
        cleanMsg = "No se pudo establecer conexión con el servidor de Lexia. Verificá que el backend de Rust esté corriendo.";
    }
    
    errorMessage.textContent = cleanMsg;
    showView('error');
}

// ── Acciones de salida (Copiar y PDF) ───────────────────────────────────────
btnCopy.addEventListener('click', async () => {
    await navigator.clipboard.writeText(state.lastResult);
    const original = btnCopy.textContent;
    btnCopy.textContent = 'Copiado';
    setTimeout(() => btnCopy.textContent = original, 1500);
});

btnDownload.addEventListener('click', downloadPDF);

function downloadPDF() {
    const { jsPDF }  = window.jspdf;
    const doc        = new jsPDF({ unit: 'mm', format: 'a4' });
    const pageW      = doc.internal.pageSize.getWidth();
    const pageH      = doc.internal.pageSize.getHeight();
    const margin     = 22;
    const contentW   = pageW - margin * 2;
    const isPuntos   = state.mode === 'puntos-clave';
    const modeLabel  = isPuntos ? 'Puntos Clave' : 'Resumen';
    const baseName   = state.lastFilename.replace(/\.[^.]+$/, '');

    // Encabezado cobalto
    doc.setFillColor(225, 29, 72); // Rojo carmín de LEXIA
    doc.rect(0, 0, pageW, 16, 'F');
    doc.setFont('helvetica', 'bold');
    doc.setFontSize(11);
    doc.setTextColor(255, 255, 255);
    doc.text('LEXIA — Análisis de documentos con IA', margin, 10.5);

    // Metadata
    doc.setFillColor(245, 245, 248);
    doc.rect(0, 16, pageW, 20, 'F');
    doc.setTextColor(30, 30, 50);
    doc.setFontSize(15);
    doc.text(modeLabel, margin, 27);
    doc.setFont('helvetica', 'normal');
    doc.setFontSize(8.5);
    doc.setTextColor(110, 110, 130);
    doc.text(`Archivo: ${state.lastFilename}`, margin, 33);

    // Regla horizontal
    doc.setDrawColor(225, 29, 72);
    doc.setLineWidth(0.4);
    doc.line(margin, 37, pageW - margin, 37);

    // Cuerpo
    let y = 46;
    doc.setFont('helvetica', 'normal');
    doc.setFontSize(10);
    doc.setTextColor(35, 35, 55);

    const lines = doc.splitTextToSize(state.lastResult, contentW);

    for (const line of lines) {
        if (y > pageH - 18) {
            doc.addPage();
            y = 20;
        }
        if (line.trim().startsWith('•')) {
            doc.setTextColor(225, 29, 72);
            doc.setFont('helvetica', 'bold');
            doc.text(line, margin, y);
            doc.setTextColor(35, 35, 55);
            doc.setFont('helvetica', 'normal');
        } else {
            doc.text(line, margin, y);
        }
        y += 5.5;
    }

    // Footer de páginas
    const total = doc.internal.getNumberOfPages();
    for (let i = 1; i <= total; i++) {
        doc.setPage(i);
        doc.setFontSize(7.5);
        doc.setTextColor(160, 160, 175);
        doc.text(
            `Generado por LEXIA · Desarrollado por Agustín Pollán · Página ${i} de ${total}`,
            pageW / 2, pageH - 7, { align: 'center' }
        );
    }

    doc.save(`${modeLabel.toLowerCase().replace(' ', '-')}-${baseName}.pdf`);
}

// ── Retorno al inicio ───────────────────────────────────────────────────────
btnNew.addEventListener('click', resetToMain);
btnRetry.addEventListener('click', resetToMain);

function resetToMain() {
    clearFile();
    showView('main');
}

// ── Utilerías de Formateo ───────────────────────────────────────────────────
function formatBytes(bytes) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
}

// Utilidad para limpiar HTML
function escapeHtml(str) {
    return str
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

// ── FONDO INTERACTIVO DE PARTÍCULAS 3D (Google Antigravity Original) ───────
(function initAntigravityBackground() {
    const canvas = $('particles-canvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');

    let width = canvas.width = window.innerWidth;
    let height = canvas.height = window.innerHeight;

    let particles = [];
    const numParticles = 260;
    const fov = 300; 
    const cameraDistance = 350;

    // Límites tridimensionales
    const boundsX = 600;
    const boundsY = 600;
    const boundsZ = 400;

    // Mouse para interacción
    const mouse = {
        targetX: 0,
        targetY: 0,
        radius: 130
    };

    // Velocidades angulares de rotación por frame
    let speedX = 0.0003; 
    let speedY = 0.0004;

    window.addEventListener('resize', () => {
        width = canvas.width = window.innerWidth;
        height = canvas.height = window.innerHeight;
    });

    window.addEventListener('mousemove', e => {
        mouse.targetX = e.clientX;
        mouse.targetY = e.clientY;

        // Modificar velocidad angular de rotación según el mouse
        speedY = (e.clientX - width / 2) * 0.000015;
        speedX = (e.clientY - height / 2) * 0.000015;
    });

    class Particle3D {
        constructor() {
            this.x = (Math.random() - 0.5) * boundsX;
            this.y = (Math.random() - 0.5) * boundsY;
            this.z = (Math.random() - 0.5) * boundsZ;

            this.offsetX = 0;
            this.offsetY = 0;

            this.radius = Math.random() * 1.5 + 0.5;
            this.color = Math.random() > 0.25 ? 'rgba(225, 29, 72, ' : 'rgba(244, 63, 94, ';
        }

        update() {
            // Asegurar una velocidad de rotación base para que nunca se frene
            const minSpeedX = speedX === 0 ? 0.0002 : speedX;
            const minSpeedY = speedY === 0 ? 0.0003 : speedY;

            const cosX = Math.cos(minSpeedX);
            const sinX = Math.sin(minSpeedX);
            const cosY = Math.cos(minSpeedY);
            const sinY = Math.sin(minSpeedY);

            // Rotar eje Y
            let x1 = this.x * cosY - this.z * sinY;
            let z1 = this.x * sinY + this.z * cosY;

            // Rotar eje X
            let y1 = this.y * cosX - z1 * sinX;
            let z2 = this.y * sinX + z1 * cosX;

            this.x = x1;
            this.y = y1;
            this.z = z2;

            // Proyección Perspectiva 3D
            const scale = fov / (cameraDistance + this.z);
            const projX = (this.x * scale) + width / 2;
            const projY = (this.y * scale) + height / 2;

            // Repulsión elástica del mouse
            const dx = projX - mouse.targetX;
            const dy = projY - mouse.targetY;
            const dist = Math.sqrt(dx * dx + dy * dy);

            if (dist < mouse.radius) {
                const force = (mouse.radius - dist) / mouse.radius;
                this.offsetX += (dx / dist) * force * 10;
                this.offsetY += (dy / dist) * force * 10;
            }

            // Amortiguación de retorno
            this.offsetX *= 0.92;
            this.offsetY *= 0.92;

            const finalX = projX + this.offsetX;
            const finalY = projY + this.offsetY;

            if (finalX >= 0 && finalX <= width && finalY >= 0 && finalY <= height) {
                const alpha = Math.max(0.12, ((this.z + boundsZ / 2) / boundsZ) * 0.75);

                ctx.beginPath();
                ctx.arc(finalX, finalY, this.radius * scale, 0, Math.PI * 2);
                ctx.fillStyle = this.color + alpha + ')';
                ctx.fill();
            }
        }
    }

    // Inicializar universo
    for (let i = 0; i < numParticles; i++) {
        particles.push(new Particle3D());
    }

    // Loop de animación
    function animate() {
        ctx.fillStyle = 'rgba(255, 255, 255, 0.22)'; // Estela en modo claro
        ctx.fillRect(0, 0, width, height);

        // Si el mouse no se mueve, mantener velocidad mínima constante
        const baseSpeedX = 0.0003;
        const baseSpeedY = 0.0004;
        
        speedX += (baseSpeedX - speedX) * 0.05;
        speedY += (baseSpeedY - speedY) * 0.05;

        particles.forEach(p => p.update());

        requestAnimationFrame(animate);
    }

    animate();
})();

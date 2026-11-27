# EndoInforme

Informes de endoscopía digestiva alta y colonoscopía en clics. Funciona sin internet.

Herramienta creada por **Luis Vélez Vizcaíno** — Internista · Residente de Gastroenterología, Universidad de los Andes.
© 2026 SALUVEL SpA. Todos los derechos reservados.

## Tres formas de usarla

### 1. En el navegador (y como app instalada)
Abre **https://saluvel.github.io/EndoInforme/** en Chrome o Edge y aprieta **Instalar** (abajo a la derecha en la pantalla de inicio).
- **Chrome / Edge (Windows o Mac):** se instala con un clic.
- **Safari en Mac (macOS 14+):** Archivo → Agregar al Dock.
- **iPhone / iPad:** Compartir → Agregar a pantalla de inicio.

### 2. Programa para Windows o Mac (pendrive o Drive)
Descarga el instalador desde **[Releases](https://github.com/Saluvel/EndoInforme/releases/latest)**:
- **Windows:** `EndoInforme_x.y.z_x64-setup.exe` → doble clic. Si aparece *"Windows protegió su PC"*, haz clic en **Más información → Ejecutar de todas formas** (la app no tiene firma de pago de Microsoft).
- **Mac:** `EndoInforme_x.y.z_universal.dmg` → doble clic y arrastra EndoInforme a **Aplicaciones**. La primera vez, si dice que *no se puede verificar el desarrollador*, ve a **Ajustes del Sistema → Privacidad y seguridad → Abrir igualmente** (o clic derecho sobre la app → Abrir).

### 3. Archivo único
`index.html` abre con doble clic en cualquier navegador, sin instalar nada.

## Privacidad
Los datos del paciente no se guardan: viven solo en la memoria de la ventana. Las plantillas y ajustes quedan en cada computador.

## Para el mantenedor
- `index.html`, `manifest.webmanifest`, `sw.js`, íconos: sitio de GitHub Pages (rama `main`, carpeta raíz).
- `desktop/`: proyecto Tauri. Crear un tag `vX.Y.Z` (o ejecutar el workflow *Instaladores de escritorio* a mano) compila el `.exe` y el `.dmg` y los publica en Releases.

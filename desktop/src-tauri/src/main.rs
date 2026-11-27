// Ventana de escritorio que carga EndoInforme (un solo HTML, sin internet).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar EndoInforme");
}

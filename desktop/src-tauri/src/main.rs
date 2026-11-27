// Ventana de escritorio que carga EndoInforme (un solo HTML, sin internet).
// Persistencia: las preferencias, plantillas, perfiles y clave (claves "endoinforme.*" del almacenamiento local)
// se guardan además en un archivo dentro de la carpeta de datos de la app, que sobrevive a cerrar/abrir y a
// actualizar de versión. Al abrir, ese archivo se inyecta antes de que cargue la página y restaura lo que falte.
// Nunca se guardan datos de pacientes (esos viven solo en memoria).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

fn ruta_datos(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("endoinforme-datos.json"))
}

#[tauri::command]
fn guardar_datos(app: AppHandle, datos: String) -> Result<(), String> {
    // solo JSON válido; escritura atómica (archivo temporal + renombrar) y respaldo del anterior
    serde_json::from_str::<serde_json::Value>(&datos).map_err(|e| e.to_string())?;
    let p = ruta_datos(&app).ok_or("sin carpeta de datos")?;
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if p.exists() {
        let _ = fs::copy(&p, p.with_extension("json.bak"));
    }
    let tmp = p.with_extension("json.tmp");
    fs::write(&tmp, datos.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &p).map_err(|e| e.to_string())
}

fn leer_datos(app: &AppHandle) -> String {
    let leer = |p: PathBuf| {
        fs::read_to_string(p)
            .ok()
            .filter(|s| serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(s).is_ok())
    };
    ruta_datos(app)
        .and_then(|p| leer(p.clone()).or_else(|| leer(p.with_extension("json.bak"))))
        .unwrap_or_else(|| "{}".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![guardar_datos])
        .setup(|app| {
            let datos = leer_datos(app.handle());
            let js = format!(
                "window.ENDO_DISCO={d};try{{var D=window.ENDO_DISCO;for(var k in D){{if(/^endoinforme\\./.test(k)&&typeof D[k]==='string'&&localStorage.getItem(k)===null)localStorage.setItem(k,D[k]);}}}}catch(e){{}}",
                d = datos
            );
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("EndoInforme")
                .inner_size(1440.0, 900.0)
                .min_inner_size(1024.0, 640.0)
                .center()
                .maximized(true)
                .initialization_script(&js)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar EndoInforme");
}

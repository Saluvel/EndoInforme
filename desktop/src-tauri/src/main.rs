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

#[tauri::command]
fn info_app() -> serde_json::Value {
    let os = match std::env::consts::OS { "macos" => "macos", "windows" => "windows", o => o };
    serde_json::json!({ "version": env!("CARGO_PKG_VERSION"), "os": os, "arch": std::env::consts::ARCH, "auto": true })
}

// Abre en el navegador del sistema SOLO descargas del repositorio oficial (actualizaciones)
#[tauri::command]
fn abrir_url(url: String) -> Result<(), String> {
    // Releases (sin caracteres especiales), o issue nuevo / correo (URL codificada: solo caracteres URL seguros, sin comillas ni shell)
    let rel = url.starts_with("https://github.com/Saluvel/EndoInforme/releases/")
        && !url.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'' || c == '&' || c == '|' || c == '^' || c == '%' || c == '`');
    let enc = (url.starts_with("https://github.com/Saluvel/EndoInforme/issues/new?") || url.starts_with("mailto:?"))
        && url.len() < 8000
        && url.chars().all(|c| c.is_ascii_alphanumeric() || "-._~:/?#@!$&()*+,;=%".contains(c));
    let ok = rel || enc;
    if !ok {
        return Err("URL no permitida".into());
    }
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", &url]).spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg(&url).spawn();
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let r = std::process::Command::new("xdg-open").arg(&url).spawn();
    r.map(|_| ()).map_err(|e| e.to_string())
}

// ---------- Actualizar en el lugar ("Actualizar y reiniciar") ----------
// Descarga el instalador del release oficial (solo github.com/Saluvel/EndoInforme), verifica su SHA-256
// (el que publica GitHub para ese archivo), lo instala sobre la versión actual y reinicia la app.
// Los datos (usuarios, plantillas, ajustes, clave) viven en la carpeta de datos y no se tocan.
fn sh(cmd: &str, args: &[&str]) -> Result<(), String> {
    let mut c = std::process::Command::new(cmd);
    c.args(args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // sin ventana de consola
    }
    let st = c.status().map_err(|e| format!("{}: {}", cmd, e))?;
    if st.success() { Ok(()) } else { Err(format!("{} terminó con error ({})", cmd, st)) }
}

fn sha256_de(p: &PathBuf) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut f = fs::File::open(p).map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 { break; }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{:02x}", b)).collect())
}

#[tauri::command]
async fn actualizar(app: AppHandle, url: String, sha256: String) -> Result<(), String> {
    let pre = "https://github.com/Saluvel/EndoInforme/releases/download/";
    let nombre = url.rsplit('/').next().unwrap_or("").to_string();
    let url_ok = url.starts_with(pre)
        && url[pre.len()..].chars().all(|c| c.is_ascii_alphanumeric() || "._-/".contains(c))
        && !url.contains("..")
        && (nombre.ends_with(".dmg") || nombre.ends_with(".exe"));
    let sha = sha256.trim().trim_start_matches("sha256:").to_ascii_lowercase();
    if !url_ok || sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Descarga no permitida".into());
    }
    let tmp = std::env::temp_dir().join("endoinforme-actualizacion");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let archivo = tmp.join(&nombre);
    let arch_s = archivo.to_string_lossy().to_string();
    #[cfg(target_os = "windows")]
    let curl = "curl.exe";
    #[cfg(not(target_os = "windows"))]
    let curl = "curl";
    sh(curl, &["-fsSL", "--retry", "2", "--proto", "=https", "-o", &arch_s, &url]).map_err(|e| format!("No se pudo descargar: {}", e))?;
    if sha256_de(&archivo)? != sha {
        let _ = fs::remove_file(&archivo);
        return Err("La descarga no coincide con la firma del archivo publicado (SHA-256). No se instaló nada.".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        let bundle = exe.ancestors().find(|p| p.extension().map(|e| e == "app").unwrap_or(false)).map(|p| p.to_path_buf()).ok_or("No se encontró EndoInforme.app")?;
        let bs = bundle.to_string_lossy().to_string();
        if bs.contains("AppTranslocation") || bs.starts_with("/Volumes/") {
            return Err("EndoInforme se está abriendo desde el .dmg o una ubicación temporal. Arrástralo a Aplicaciones y vuelve a intentarlo.".into());
        }
        let mnt = tmp.join("mnt");
        fs::create_dir_all(&mnt).map_err(|e| e.to_string())?;
        let mnt_s = mnt.to_string_lossy().to_string();
        sh("hdiutil", &["attach", "-nobrowse", "-noautoopen", "-quiet", "-mountpoint", &mnt_s, &arch_s])?;
        let nueva = fs::read_dir(&mnt).ok().and_then(|it| it.filter_map(|e| e.ok()).map(|e| e.path()).find(|p| p.extension().map(|e| e == "app").unwrap_or(false)));
        let res = (|| -> Result<(), String> {
            let nueva = nueva.ok_or("El .dmg no trae la app")?;
            let staging = bundle.with_extension("app-nueva");
            let vieja = bundle.with_extension("app-anterior");
            let _ = fs::remove_dir_all(&staging);
            let _ = fs::remove_dir_all(&vieja);
            let (n_s, st_s) = (nueva.to_string_lossy().to_string(), staging.to_string_lossy().to_string());
            sh("ditto", &[&n_s, &st_s]).map_err(|e| format!("No se pudo copiar (¿permisos en Aplicaciones?): {}", e))?;
            let _ = sh("xattr", &["-dr", "com.apple.quarantine", &st_s]);
            fs::rename(&bundle, &vieja).map_err(|e| format!("No se pudo reemplazar la app: {}", e))?;
            if let Err(e) = fs::rename(&staging, &bundle) {
                let _ = fs::rename(&vieja, &bundle); // vuelve atrás: la versión actual queda intacta
                return Err(format!("No se pudo reemplazar la app: {}", e));
            }
            let _ = fs::remove_dir_all(&vieja);
            Ok(())
        })();
        let _ = sh("hdiutil", &["detach", "-quiet", "-force", &mnt_s]);
        res?;
        let _ = fs::remove_dir_all(&tmp);
        std::process::Command::new("/bin/sh").args(["-c", "sleep 1; open -n \"$0\"", &bs]).spawn().map_err(|e| e.to_string())?;
        app.exit(0);
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let ok_ruta = |s: &str| !s.contains('"') && !s.contains('%') && !s.contains('^') && !s.contains('&') && !s.contains('|');
        let exe_s = exe.to_string_lossy().to_string();
        if !ok_ruta(&arch_s) || !ok_ruta(&exe_s) {
            return Err("Ruta con caracteres no soportados".into());
        }
        // espera a que la app se cierre, instala en silencio sobre la actual y la vuelve a abrir
        let linea = format!("/C ping -n 3 127.0.0.1 >nul & \"{}\" /S & start \"\" \"{}\"", arch_s, exe_s);
        std::process::Command::new("cmd").raw_arg(linea).creation_flags(0x0800_0000).spawn().map_err(|e| e.to_string())?;
        app.exit(0);
        return Ok(());
    }

    #[allow(unreachable_code)]
    {
        let _ = (app, exe);
        Err("Actualización automática no disponible en este sistema".into())
    }
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
        .invoke_handler(tauri::generate_handler![guardar_datos, info_app, abrir_url, actualizar])
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
        .on_window_event(|window, event| {
            // al cerrar: pedir a la página que guarde la última copia a disco y cerrar un instante después
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(w) = window.app_handle().get_webview_window(window.label()) {
                    if w.eval("try{window.EndoDisco&&window.EndoDisco.save()}catch(e){}").is_ok() {
                        api.prevent_close();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(350));
                            let _ = w.destroy();
                        });
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar EndoInforme");
}

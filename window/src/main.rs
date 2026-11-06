#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! Lanzador gráfico sin consola; los controles y el host continúan separados.
fn main() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Start-Boddy.ps1");
        let _ = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-STA", "-File"])
            .arg(script)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW: sin ocultar la ventana WinForms.
            .spawn();
    }
    #[cfg(not(windows))]
    eprintln!("La interfaz de Boddy requiere Windows.");
}

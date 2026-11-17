// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Chromium relance cet exécutable pour ses processus de rendu, GPU et utilitaires
    // (`--type=renderer`…). Ils passent la main à CEF tout de suite, avant le journal,
    // la base et tokio : seul le processus principal démarre l'application.
    if std::env::args().any(|arg| arg.starts_with("--type=")) {
        tauri_runtime_cef::run_cef_helper_process();
        return;
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime tokio")
        .block_on(rustmusic_lib::run());
}

#[tauri::command]
fn core_diagnostic() -> String {
    format!(
        "Core Rust ativo • DIAMANDIS ONE v{}",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![core_diagnostic])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar DIAMANDIS ONE");
}
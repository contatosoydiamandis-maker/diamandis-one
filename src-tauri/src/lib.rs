mod file_ops;
mod gamehub;
mod settings;

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::io::ErrorKind;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileEntry { name: String, path: String, is_dir: bool, size: Option<u64> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryListing { path: String, parent: Option<String>, entries: Vec<FileEntry> }

#[tauri::command]
fn core_diagnostic() -> String {
    format!("Core Rust ativo • DIAMANDIS ONE v{}", env!("CARGO_PKG_VERSION"))
}

#[tauri::command]
async fn list_directory(path: Option<String>) -> Result<DirectoryListing, String> {
    tauri::async_runtime::spawn_blocking(move || list_directory_impl(path)).await
        .map_err(|_| "Não foi possível carregar esta pasta em segundo plano.".to_string())?
}

fn list_directory_impl(path: Option<String>) -> Result<DirectoryListing, String> {
    let requested = path.filter(|v| !v.trim().is_empty()).map(PathBuf::from).unwrap_or_else(home_dir);
    let canonical = requested.canonicalize().map_err(|e| friendly_io_error("acessar esta pasta", &e))?;
    if !canonical.is_dir() { return Err("O caminho informado não é uma pasta.".into()); }
    let iterator = fs::read_dir(&canonical).map_err(|e| friendly_io_error("listar esta pasta", &e))?;
    let mut entries = Vec::new();
    for item in iterator {
        let item = item.map_err(|e| friendly_io_error("ler um item desta pasta", &e))?;
        let item_path = item.path();
        let metadata = item.metadata().ok();
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        entries.push(FileEntry {
            name: item.file_name().to_string_lossy().to_string(),
            path: item_path.to_string_lossy().to_string(),
            is_dir,
            size: metadata.filter(|m| m.is_file()).map(|m| m.len()),
        });
    }
    entries.sort_by(|a,b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(DirectoryListing {
        path: canonical.to_string_lossy().to_string(),
        parent: canonical.parent().map(|p| p.to_string_lossy().to_string()),
        entries,
    })
}

#[tauri::command]
async fn create_directory(parent: String, name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || create_directory_impl(parent, name)).await
        .map_err(|_| "Não foi possível criar a pasta em segundo plano.".to_string())?
}

fn create_directory_impl(parent: String, name: String) -> Result<(), String> {
    let name = file_ops::validate_name(&name)?;
    let parent = PathBuf::from(parent).canonicalize().map_err(|e| friendly_io_error("acessar a pasta de destino", &e))?;
    if !parent.is_dir() { return Err("O destino informado não é uma pasta.".into()); }
    let destination = parent.join(name);
    if destination.exists() { return Err("Já existe um item com esse nome.".into()); }
    fs::create_dir(destination).map_err(|e| friendly_io_error("criar a pasta", &e))
}


fn friendly_io_error(action: &str, error: &std::io::Error) -> String {
    let reason = match error.kind() {
        ErrorKind::NotFound => "o local não foi encontrado",
        ErrorKind::PermissionDenied => "o Windows não permitiu o acesso. Verifique as permissões",
        ErrorKind::AlreadyExists => "já existe um item com esse nome",
        ErrorKind::InvalidInput | ErrorKind::InvalidData => "o caminho ou os dados informados são inválidos",
        _ => "ocorreu um erro inesperado",
    };
    format!("Não foi possível {action}: {reason}.")
}


fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![core_diagnostic, list_directory, create_directory, file_ops::rename_item, file_ops::copy_file, file_ops::move_item, file_ops::delete_item, settings::load_settings, settings::save_settings, gamehub::scan_games])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar DIAMANDIS ONE");
}
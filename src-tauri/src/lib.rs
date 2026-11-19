use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};

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
fn list_directory(path: Option<String>) -> Result<DirectoryListing, String> {
    let requested = path.filter(|v| !v.trim().is_empty()).map(PathBuf::from).unwrap_or_else(home_dir);
    let canonical = requested.canonicalize().map_err(|e| format!("Não foi possível acessar esta pasta: {e}"))?;
    if !canonical.is_dir() { return Err("O caminho informado não é uma pasta.".into()); }
    let iterator = fs::read_dir(&canonical).map_err(|e| format!("Não foi possível listar esta pasta: {e}"))?;
    let mut entries = Vec::new();
    for item in iterator {
        let item = item.map_err(|e| format!("Falha ao ler um item: {e}"))?;
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
fn create_directory(parent: String, name: String) -> Result<(), String> {
    let name = name.trim();
    let relative = Path::new(name);
    if name.is_empty() || !matches!(relative.components().next(), Some(Component::Normal(_))) || relative.components().count() != 1 {
        return Err("Informe um nome de pasta válido, sem caminhos.".into());
    }
    let parent = PathBuf::from(parent).canonicalize().map_err(|e| format!("Não foi possível acessar o destino: {e}"))?;
    if !parent.is_dir() { return Err("O destino informado não é uma pasta.".into()); }
    let destination = parent.join(name);
    if destination.exists() { return Err("Já existe um item com esse nome.".into()); }
    fs::create_dir(destination).map_err(|e| format!("Não foi possível criar a pasta: {e}"))
}

fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![core_diagnostic, list_directory, create_directory])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar DIAMANDIS ONE");
}
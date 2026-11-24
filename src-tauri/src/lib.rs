mod atomic_file;
mod file_ops;
mod gamehub;
mod settings;
mod save_center;
mod totk_recomp;
mod system_metrics;

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::io::ErrorKind;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileEntry { name: String, path: String, is_dir: bool, size: Option<u64> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryListing {
    path: String,
    parent: Option<String>,
    total: usize,
    offset: usize,
    limit: usize,
    entries: Vec<FileEntry>,
}

#[tauri::command]
fn core_diagnostic() -> String {
    format!("Core Rust ativo • DIAMANDIS ONE v{}", env!("CARGO_PKG_VERSION"))
}

#[tauri::command]
async fn list_directory(
    path: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
    query: Option<String>,
    kind: Option<String>,
    sort: Option<String>,
) -> Result<DirectoryListing, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(150).clamp(25, 500);
    let query = query.unwrap_or_default();
    let kind = kind.unwrap_or_else(|| "all".into());
    let sort = sort.unwrap_or_else(|| "name".into());
    tauri::async_runtime::spawn_blocking(move || {
        list_directory_impl(path, offset, limit, query, kind, sort)
    }).await.map_err(|_| "Não foi possível carregar esta pasta em segundo plano.".to_string())?
}

fn file_type_sort_key(name: &str) -> String {
    PathBuf::from(name).extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn matches_directory_filters(name: &str, is_dir: bool, query: &str, kind: &str) -> bool {
    let matches_query = query.is_empty() || name.to_lowercase().contains(query);
    let matches_kind = match kind {
        "folders" => is_dir,
        "files" => !is_dir,
        _ => true,
    };
    matches_query && matches_kind
}

fn list_directory_impl(
    path: Option<String>,
    offset: usize,
    limit: usize,
    query: String,
    kind: String,
    sort: String,
) -> Result<DirectoryListing, String> {
    if query.len() > 512 { return Err("O filtro de busca excede o limite permitido.".into()); }
    if !matches!(kind.as_str(), "all" | "folders" | "files") {
        return Err("O filtro de tipo informado é inválido.".into());
    }
    if !matches!(sort.as_str(), "name" | "size" | "type") {
        return Err("A ordenação informada é inválida.".into());
    }
    let query = query.trim().to_lowercase();
    let requested = path.filter(|v| !v.trim().is_empty()).map(PathBuf::from).unwrap_or_else(home_dir);
    let canonical = requested.canonicalize().map_err(|e| friendly_io_error("acessar esta pasta", &e))?;
    if !canonical.is_dir() { return Err("O caminho informado não é uma pasta.".into()); }
    let iterator = fs::read_dir(&canonical).map_err(|e| friendly_io_error("listar esta pasta", &e))?;
    let mut candidates = Vec::new();
    for item in iterator {
        let item = item.map_err(|e| friendly_io_error("ler um item desta pasta", &e))?;
        let item_path = item.path();
        let file_type = item.file_type().map_err(|e| friendly_io_error("identificar um item desta pasta", &e))?;
        let name = item.file_name().to_string_lossy().to_string();
        let is_dir = file_type.is_dir();
        let is_symlink = file_type.is_symlink();
        if !matches_directory_filters(&name, is_dir, &query, &kind) { continue; }
        let size = if !is_dir && !is_symlink {
            fs::metadata(&item_path).ok().filter(|metadata| metadata.is_file()).map(|metadata| metadata.len())
        } else { None };
        candidates.push((name, item_path, is_dir, size));
    }
    candidates.sort_by(|a, b| match sort.as_str() {
        "size" => b.3.unwrap_or(0).cmp(&a.3.unwrap_or(0)).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())),
        "type" => b.2.cmp(&a.2)
            .then_with(|| file_type_sort_key(&a.0).cmp(&file_type_sort_key(&b.0)))
            .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())),
        _ => b.2.cmp(&a.2).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())),
    });
    let total = candidates.len();
    let safe_offset = offset.min(total);
    let entries = candidates.into_iter().skip(safe_offset).take(limit).map(|(name, item_path, is_dir, size)| {
        FileEntry { name, path: item_path.to_string_lossy().to_string(), is_dir, size }
    }).collect();
    Ok(DirectoryListing {
        path: canonical.to_string_lossy().to_string(),
        parent: canonical.parent().map(|p| p.to_string_lossy().to_string()),
        total,
        offset: safe_offset,
        limit,
        entries,
    })
}

#[cfg(test)]
mod directory_filter_tests {
    use super::{file_type_sort_key, matches_directory_filters};

    #[test]
    fn sorts_file_extensions_case_insensitively() {
        assert_eq!(file_type_sort_key("save.DAT"), "dat");
        assert_eq!(file_type_sort_key("archive.zip"), "zip");
        assert_eq!(file_type_sort_key("README"), "");
    }

    #[test]
    fn searches_names_case_insensitively_across_file_types() {
        assert!(matches_directory_filters("SaveGame.DAT", false, "save", "all"));
        assert!(!matches_directory_filters("Other.dat", false, "save", "all"));
    }

    #[test]
    fn applies_folder_and_file_filters() {
        assert!(matches_directory_filters("Saves", true, "", "folders"));
        assert!(!matches_directory_filters("save.dat", false, "", "folders"));
        assert!(matches_directory_filters("save.dat", false, "", "files"));
        assert!(!matches_directory_filters("Saves", true, "", "files"));
    }
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
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![core_diagnostic, system_metrics::get_system_metrics, list_directory, create_directory, file_ops::rename_item, file_ops::copy_file, file_ops::move_item, file_ops::delete_item, settings::load_settings, settings::save_settings, gamehub::scan_games, gamehub::open_game_folder, gamehub::launch_game, save_center::list_save_games, save_center::add_save_game, save_center::remove_save_game, save_center::create_save_backup, save_center::list_save_backups, save_center::verify_save_backup, save_center::restore_save_backup, save_center::export_save_backup, save_center::delete_save_backup, totk_recomp::configure_totk_install, totk_recomp::get_totk_install, totk_recomp::list_totk_campaigns, totk_recomp::import_totk_profile, totk_recomp::create_empty_totk_campaign, totk_recomp::launch_totk_campaign, totk_recomp::is_totk_running])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar DIAMANDIS ONE");
}

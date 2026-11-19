use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
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
fn list_directory(path: Option<String>) -> Result<DirectoryListing, String> {
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
fn create_directory(parent: String, name: String) -> Result<(), String> {
    let name = validate_folder_name(&name)?;
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

fn validate_folder_name(name: &str) -> Result<&str, String> {
    let clean = name.trim();
    let candidate = Path::new(clean);
    let reserved_stem = clean.split('.').next().unwrap_or("").trim().to_ascii_uppercase();
    let reserved_device = matches!(reserved_stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (["COM", "LPT"].iter().any(|prefix| {
            reserved_stem.strip_prefix(prefix)
                .and_then(|suffix| suffix.parse::<u8>().ok())
                .is_some_and(|number| (1..=9).contains(&number))
        }));
    let invalid_windows_character = clean.chars().any(|ch| {
        ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
    });
    if clean.is_empty()
        || clean.ends_with(' ') || clean.ends_with('.')
        || invalid_windows_character
        || reserved_device
        || !matches!(candidate.components().next(), Some(Component::Normal(_)))
        || candidate.components().count() != 1
    {
        return Err("Use um nome de pasta válido para o Windows. Evite caminhos, caracteres especiais e nomes reservados.".into());
    }
    Ok(clean)
}

#[cfg(test)]
mod tests {
    use super::validate_folder_name;

    #[test]
    fn accepts_simple_folder_names() {
        assert_eq!(validate_folder_name("Minhas imagens").unwrap(), "Minhas imagens");
        assert_eq!(validate_folder_name("  Projetos  ").unwrap(), "Projetos");
    }

    #[test]
    fn rejects_empty_and_traversal_names() {
        assert!(validate_folder_name("").is_err());
        assert!(validate_folder_name("   ").is_err());
        assert!(validate_folder_name("..").is_err());
        assert!(validate_folder_name("../fora").is_err());
        assert!(validate_folder_name("pasta/subpasta").is_err());
        assert!(validate_folder_name("pasta\\\\subpasta").is_err());
        assert!(validate_folder_name("CON").is_err());
        assert!(validate_folder_name("NUL.txt").is_err());
        assert!(validate_folder_name("nome.").is_err());
        assert!(validate_folder_name("nome?invalido").is_err());
    }
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
use serde::Serialize;
use std::{collections::HashSet, env, fs, path::PathBuf, process::Command};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEntry {
    pub name: String,
    pub source: String,
    pub path: String,
}

fn env_path(key: &str) -> Option<PathBuf> { env::var_os(key).map(PathBuf::from) }

fn add_game(games: &mut Vec<GameEntry>, seen: &mut HashSet<String>, name: String, source: &str, path: PathBuf) {
    if !path.is_dir() { return; }
    let key = path.to_string_lossy().to_lowercase();
    if !seen.insert(key) { return; }
    games.push(GameEntry { name, source: source.into(), path: path.to_string_lossy().into_owned() });
}

fn quoted_value(line: &str, key: &str) -> Option<String> {
    let mut parts = line.split('"').filter(|part| !part.trim().is_empty());
    let found = parts.next()?;
    if found != key { return None; }
    parts.next().map(|value| value.to_string())
}

fn scan_steam(games: &mut Vec<GameEntry>, seen: &mut HashSet<String>) {
    let mut roots = Vec::new();
    if let Some(local) = env_path("PROGRAMFILES(X86)") { roots.push(local.join("Steam")); }
    if let Some(local) = env_path("PROGRAMFILES") { roots.push(local.join("Steam")); }
    if let Some(local) = env_path("LOCALAPPDATA") { roots.push(local.join("Programs/Steam")); }
    for root in roots {
        if !root.exists() { continue; }
        let mut libraries = vec![root.clone()];
        let config = root.join("steamapps/libraryfolders.vdf");
        if let Ok(contents) = fs::read_to_string(config) {
            for line in contents.lines() {
                if let Some(value) = quoted_value(line.trim(), "path") {
                    libraries.push(PathBuf::from(value.replace("\\\\", "\\")));
                }
            }
        }
        for library in libraries {
            let steamapps = library.join("steamapps");
            let Ok(entries) = fs::read_dir(&steamapps) else { continue };
            for entry in entries.flatten() {
                let file = entry.path();
                if file.extension().and_then(|v| v.to_str()) != Some("acf") { continue; }
                let Ok(text) = fs::read_to_string(&file) else { continue };
                let mut name = None;
                let mut install_dir = None;
                for line in text.lines() {
                    let line = line.trim();
                    if name.is_none() { name = quoted_value(line, "name"); }
                    if install_dir.is_none() { install_dir = quoted_value(line, "installdir"); }
                }
                if let (Some(name), Some(install_dir)) = (name, install_dir) {
                    add_game(games, seen, name, "Steam", steamapps.join("common").join(install_dir));
                }
            }
        }
    }
}

fn scan_epic(games: &mut Vec<GameEntry>, seen: &mut HashSet<String>) {
    let Some(program_data) = env_path("PROGRAMDATA") else { return };
    let manifests = program_data.join("Epic/EpicGamesLauncher/Data/Manifests");
    let Ok(entries) = fs::read_dir(manifests) else { return };
    for entry in entries.flatten() {
        if entry.path().extension().and_then(|v| v.to_str()) != Some("item") { continue; }
        let Ok(contents) = fs::read_to_string(entry.path()) else { continue };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) else { continue };
        let name = value.get("DisplayName").and_then(|v| v.as_str()).unwrap_or("Jogo Epic").to_string();
        let install = value.get("InstallLocation").and_then(|v| v.as_str()).unwrap_or("");
        if !install.is_empty() { add_game(games, seen, name, "Epic Games", PathBuf::from(install)); }
    }
}

fn scan_gog(games: &mut Vec<GameEntry>, seen: &mut HashSet<String>) {
    let mut roots = Vec::new();
    if let Some(program_files) = env_path("PROGRAMFILES(X86)") { roots.push(program_files.join("GOG Galaxy/Games")); }
    if let Some(program_files) = env_path("PROGRAMFILES") { roots.push(program_files.join("GOG Galaxy/Games")); }
    if let Some(system_drive) = env_path("SYSTEMDRIVE") { roots.push(system_drive.join("GOG Games")); }
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else { continue };
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                add_game(games, seen, name, "GOG", entry.path());
            }
        }
    }
}

#[tauri::command]
pub async fn scan_games() -> Result<Vec<GameEntry>, String> {
    tauri::async_runtime::spawn_blocking(scan_games_impl).await
        .map_err(|_| "Não foi possível concluir a busca de jogos em segundo plano.".to_string())?
}

fn scan_games_impl() -> Result<Vec<GameEntry>, String> {
    let mut games = Vec::new();
    let mut seen = HashSet::new();
    scan_steam(&mut games, &mut seen);
    scan_epic(&mut games, &mut seen);
    scan_gog(&mut games, &mut seen);
    games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::quoted_value;

    #[test]
    fn reads_quoted_vdf_values() {
        assert_eq!(quoted_value(r#""path" "D:\\Games\\Steam""#, "path"), Some(r#"D:\\Games\\Steam"#.into()));
        assert_eq!(quoted_value(r#""name" "Portal 2""#, "name"), Some("Portal 2".into()));
    }

    #[test]
    fn ignores_other_vdf_keys() {
        assert_eq!(quoted_value(r#""appid" "620""#, "path"), None);
    }
}


fn resolve_game_folder(path: &str) -> Result<PathBuf, String> {
    let requested = PathBuf::from(path);
    let canonical = requested.canonicalize()
        .map_err(|_| "A pasta de instalação não existe ou não pode ser acessada.".to_string())?;
    if !canonical.is_dir() {
        return Err("O caminho do jogo não aponta para uma pasta.".into());
    }
    Ok(canonical)
}

#[tauri::command]
pub async fn open_game_folder(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let folder = resolve_game_folder(&path)?;
        #[cfg(target_os = "windows")]
        let mut command = Command::new("explorer.exe");
        #[cfg(target_os = "macos")]
        let mut command = Command::new("open");
        #[cfg(all(unix, not(target_os = "macos")))]
        let mut command = Command::new("xdg-open");
        #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
        return Err("Abrir pastas não é compatível com este sistema.".into());

        command.arg(folder).spawn()
            .map_err(|_| "Não foi possível abrir a pasta do jogo no gerenciador de arquivos.".to_string())?;
        Ok(())
    }).await.map_err(|_| "Não foi possível abrir a pasta em segundo plano.".to_string())?
}

use serde::Serialize;
use std::{collections::HashSet, env, fs, path::PathBuf};

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
pub fn scan_games() -> Result<Vec<GameEntry>, String> {
    let mut games = Vec::new();
    let mut seen = HashSet::new();
    scan_steam(&mut games, &mut seen);
    scan_epic(&mut games, &mut seen);
    scan_gog(&mut games, &mut seen);
    games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(games)
}

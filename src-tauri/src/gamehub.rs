use serde::Serialize;
use serde::Deserialize;
use std::{collections::HashSet, env, fs, path::{Path, PathBuf}, process::Command};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomLibraryRoot {
    pub path: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEntry {
    pub name: String,
    pub source: String,
    pub path: String,
}

fn env_path(key: &str) -> Option<PathBuf> { env::var_os(key).map(PathBuf::from) }

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

fn add_game(games: &mut Vec<GameEntry>, seen: &mut HashSet<String>, name: String, source: &str, path: PathBuf) {
    // Canonical paths collapse aliases such as "..", mixed separators and
    // case variants before de-duplication, which is important on Windows.
    let Ok(metadata) = fs::symlink_metadata(&path) else { return };
    if metadata.file_type().is_symlink() || !metadata.is_dir() { return; }
    let Ok(canonical) = path.canonicalize() else { return };
    let key = canonical.to_string_lossy().replace('\\', "/").to_lowercase();
    if !seen.insert(key) { return; }
    games.push(GameEntry { name, source: source.into(), path: canonical.to_string_lossy().into_owned() });
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
                if !has_extension(&file, "acf") { continue; }
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
        if !has_extension(&entry.path(), "item") { continue; }
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
pub async fn scan_games(custom_roots: Option<Vec<CustomLibraryRoot>>) -> Result<Vec<GameEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || scan_games_impl(custom_roots.unwrap_or_default())).await
        .map_err(|_| "Não foi possível concluir a busca de jogos em segundo plano.".to_string())?
}

fn directory_contains_executable(path: &Path, depth: usize, budget: &mut usize) -> bool {
    if depth > 4 || *budget == 0 { return false; }
    let Ok(entries) = fs::read_dir(path) else { return false };
    for entry in entries {
        if *budget == 0 { break; }
        *budget -= 1;
        let Ok(entry) = entry else { continue };
        let item_path = entry.path();
        let Ok(kind) = entry.file_type() else { continue };
        if kind.is_symlink() { continue; }
        if kind.is_file() && item_path.extension().and_then(|v| v.to_str()).is_some_and(|v| v.eq_ignore_ascii_case("exe")) {
            return true;
        }
        if kind.is_dir() && depth < 4 && directory_contains_executable(&item_path, depth + 1, budget) {
            return true;
        }
    }
    false
}

fn scan_library_root(games: &mut Vec<GameEntry>, seen: &mut HashSet<String>, root: &Path, source: &str) {
    let Ok(root) = root.canonicalize() else { return };
    if !root.is_dir() { return; }
    let Ok(entries) = fs::read_dir(&root) else { return };
    // Limites evitam que uma pasta de biblioteca gigantesca monopolize a interface.
    let mut root_budget = 2048usize;
    for entry in entries {
        if root_budget == 0 { break; }
        root_budget -= 1;
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let Ok(kind) = entry.file_type() else { continue };
        // Não seguir diretórios simbólicos ao varrer bibliotecas selecionadas.
        if !kind.is_dir() || kind.is_symlink() { continue; }
        let mut game_budget = 1024usize;
        if directory_contains_executable(&path, 0, &mut game_budget) {
            let name = entry.file_name().to_string_lossy().into_owned();
            add_game(games, seen, name, source, path);
        }
    }
}

fn scan_xbox_games(games: &mut Vec<GameEntry>, seen: &mut HashSet<String>) {
    // O layout acessível do Xbox/GDK usa XboxGames na raiz do volume.
    // WindowsApps permanece protegido e não é alterado nem tomado por posse.
    for letter in b'A'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\XboxGames", letter as char));
        if root.is_dir() {
            scan_library_root(games, seen, &root, "Xbox");
        }
    }
}

fn scan_games_impl(custom_roots: Vec<CustomLibraryRoot>) -> Result<Vec<GameEntry>, String> {
    let mut games = Vec::new();
    let mut seen = HashSet::new();
    scan_steam(&mut games, &mut seen);
    scan_epic(&mut games, &mut seen);
    scan_gog(&mut games, &mut seen);
    scan_xbox_games(&mut games, &mut seen);
    for root in custom_roots.into_iter().take(32) {
        let source = match root.source.as_str() {
            "Hydra" => "Hydra",
            "Xbox" => "Xbox",
            _ => "Outros",
        };
        scan_library_root(&mut games, &mut seen, Path::new(&root.path), source);
    }
    games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::{has_extension, quoted_value, resolve_game_folder, HashSet};
    use std::{fs, path::Path, time::{SystemTime, UNIX_EPOCH}};


    #[test]
    fn recognizes_launcher_manifest_extensions_case_insensitively() {
        assert!(has_extension(Path::new("appmanifest_123.ACF"), "acf"));
        assert!(has_extension(Path::new("game.ITEM"), "item"));
        assert!(!has_extension(Path::new("game.item.bak"), "item"));
        assert!(!has_extension(Path::new("README"), "acf"));
    }

    #[test]
    fn deduplicates_games_by_canonical_path() {
        let root = std::env::temp_dir().join(format!(
            "diamandis-gamehub-dedupe-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let alias = root.join("nested").join("..");
        fs::create_dir_all(root.join("nested")).unwrap();
        let mut games = Vec::new();
        let mut seen = HashSet::new();
        super::add_game(&mut games, &mut seen, "Game".into(), "Steam", root.clone());
        super::add_game(&mut games, &mut seen, "Game alias".into(), "Steam", alias);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].path, root.canonicalize().unwrap().to_string_lossy());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reads_quoted_vdf_values() {
        assert_eq!(quoted_value(r#""path" "D:\\Games\\Steam""#, "path"), Some(r#"D:\\Games\\Steam"#.into()));
        assert_eq!(quoted_value(r#""name" "Portal 2""#, "name"), Some("Portal 2".into()));
    }

    #[test]
    fn ignores_other_vdf_keys() {
        assert_eq!(quoted_value(r#""appid" "620""#, "path"), None);
    }

    #[test]
    fn accepts_an_existing_installation_folder() {
        let folder = std::env::temp_dir().join(format!(
            "diamandis-gamehub-test-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&folder).unwrap();
        let resolved = resolve_game_folder(folder.to_str().unwrap()).unwrap();
        assert_eq!(resolved, folder.canonicalize().unwrap());
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn accepts_only_executables_inside_the_selected_game_folder() {
        let root = std::env::temp_dir().join(format!(
            "diamandis-gamehub-launch-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        let game = root.join("Game");
        fs::create_dir_all(&game).unwrap();
        let executable = game.join("game.exe");
        fs::write(&executable, b"test").unwrap();
        let (resolved_folder, resolved_executable) = super::resolve_game_executable(
            game.to_str().unwrap(),
            executable.to_str().unwrap()
        ).unwrap();
        assert_eq!(resolved_folder, game.canonicalize().unwrap());
        assert_eq!(resolved_executable, executable.canonicalize().unwrap());
        let outside = root.join("outside.exe");
        fs::write(&outside, b"test").unwrap();
        assert!(super::resolve_game_executable(game.to_str().unwrap(), outside.to_str().unwrap()).is_err());
        let non_executable = game.join("readme.txt");
        fs::write(&non_executable, b"test").unwrap();
        assert!(super::resolve_game_executable(game.to_str().unwrap(), non_executable.to_str().unwrap()).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_a_missing_installation_folder() {
        let missing = std::env::temp_dir().join(format!(
            "diamandis-gamehub-missing-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        assert!(resolve_game_folder(missing.to_str().unwrap()).is_err());
    }
    #[test]
    fn detects_executables_in_nested_game_directories() {
        let root = std::env::temp_dir().join(format!(
            "diamandis-gamehub-library-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        let nested = root.join("Sample Game").join("Content");
        fs::create_dir_all(&nested).unwrap();
        let mut budget = 1024;
        assert!(!super::directory_contains_executable(&root.join("Sample Game"), 0, &mut budget));
        fs::write(nested.join("sample.exe"), b"test").unwrap();
        let mut budget = 1024;
        assert!(super::directory_contains_executable(&root.join("Sample Game"), 0, &mut budget));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executable_search_respects_its_entry_budget() {
        let root = std::env::temp_dir().join(format!(
            "diamandis-gamehub-budget-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("game.exe"), b"test").unwrap();
        let mut budget = 0;
        assert!(!super::directory_contains_executable(&root, 0, &mut budget));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn library_scanner_keeps_source_and_ignores_folders_without_executables() {
        let root = std::env::temp_dir().join(format!(
            "diamandis-gamehub-root-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        let game = root.join("Hydra Game");
        let empty = root.join("Not a game");
        fs::create_dir_all(&game).unwrap();
        fs::create_dir_all(&empty).unwrap();
        fs::write(game.join("game.exe"), b"test").unwrap();
        let mut games = Vec::new();
        let mut seen = HashSet::new();
        super::scan_library_root(&mut games, &mut seen, &root, "Hydra");
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Hydra Game");
        assert_eq!(games[0].source, "Hydra");
        fs::remove_dir_all(root).unwrap();
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


fn resolve_game_executable(folder_path: &str, executable_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let folder = PathBuf::from(folder_path).canonicalize()
        .map_err(|_| "A pasta de instalação não existe ou não pode ser acessada.".to_string())?;
    if !folder.is_dir() {
        return Err("O caminho do jogo não aponta para uma pasta.".into());
    }
    let executable = PathBuf::from(executable_path).canonicalize()
        .map_err(|_| "O executável selecionado não existe ou não pode ser acessado.".to_string())?;
    if !executable.is_file() || !executable.starts_with(&folder) {
        return Err("Por segurança, selecione um executável que esteja dentro da pasta deste jogo.".into());
    }
    if !executable.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("exe")) {
        return Err("O GameHub só inicia arquivos executáveis .exe no Windows.".into());
    }
    Ok((folder, executable))
}

#[tauri::command]
pub async fn launch_game(game_folder_path: String, executable_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (folder, executable) = resolve_game_executable(&game_folder_path, &executable_path)?;
        #[cfg(target_os = "windows")]
        {
            Command::new(&executable)
                .current_dir(&folder)
                .spawn()
                .map_err(|_| "O Windows não conseguiu iniciar este jogo. Verifique se o executável ainda existe.".to_string())?;
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (folder, executable);
            Err("A inicialização direta de jogos está disponível inicialmente no Windows.".into())
        }
    }).await.map_err(|_| "Não foi possível iniciar o jogo em segundo plano.".to_string())?
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

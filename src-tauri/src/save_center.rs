use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGame {
    pub id: String,
    pub name: String,
    pub save_path: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBackup {
    pub id: String,
    pub game_id: String,
    pub game_name: String,
    pub created_at: String,
    pub source_path: String,
    pub size_bytes: u64,
    pub file_count: u64,
    pub backup_path: String,
}

fn stamp() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().to_string()
}

fn unique_id() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos().to_string()
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let root = app.path().app_local_data_dir()
        .map_err(|_| "Não foi possível localizar os dados locais do aplicativo.".to_string())?
        .join("save-center");
    fs::create_dir_all(&root).map_err(|_| "Não foi possível preparar o armazenamento do Save Center.".to_string())?;
    Ok(root)
}

fn games_file(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join("games.json"))
}

fn backup_root(app: &AppHandle) -> Result<PathBuf, String> {
    let root = data_dir(app)?.join("backups");
    fs::create_dir_all(&root).map_err(|_| "Não foi possível preparar a pasta de backups.".to_string())?;
    Ok(root)
}

fn read_games(app: &AppHandle) -> Result<Vec<SaveGame>, String> {
    let path = games_file(app)?;
    if !path.exists() { return Ok(Vec::new()); }
    let raw = fs::read_to_string(path).map_err(|_| "Não foi possível ler a biblioteca do Save Center.".to_string())?;
    serde_json::from_str(&raw).map_err(|_| "O cadastro de jogos está inválido. Seus backups não foram alterados.".to_string())
}

fn write_games(app: &AppHandle, games: &[SaveGame]) -> Result<(), String> {
    let path = games_file(app)?;
    let bytes = serde_json::to_vec_pretty(games).map_err(|_| "Não foi possível preparar o cadastro.".to_string())?;
    let mut file = fs::File::create(&path).map_err(|_| "Não foi possível gravar o cadastro de jogos.".to_string())?;
    use std::io::Write;
    file.write_all(&bytes).and_then(|_| file.sync_all()).map_err(|_| "Não foi possível concluir a atualização do cadastro.".to_string())
}

#[tauri::command]
pub async fn list_save_games(app: AppHandle) -> Result<Vec<SaveGame>, String> {
    tauri::async_runtime::spawn_blocking(move || read_games(&app)).await
        .map_err(|_| "Falha ao carregar os jogos.".to_string())?
}

#[tauri::command]
pub async fn add_save_game(app: AppHandle, name: String, save_path: String) -> Result<SaveGame, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let name = name.trim();
        if name.is_empty() || name.len() > 120 { return Err("Informe um nome de jogo válido (até 120 caracteres).".into()); }
        if save_path.trim().is_empty() || save_path.len() > 32_000 { return Err("Informe uma pasta de save válida.".into()); }
        let path = PathBuf::from(save_path).canonicalize().map_err(|_| "A pasta informada não existe ou não pode ser acessada.".to_string())?;
        if !path.is_dir() { return Err("O local informado não é uma pasta.".into()); }
        let mut games = read_games(&app)?;
        if games.iter().any(|g| PathBuf::from(&g.save_path) == path) {
            return Err("Esta pasta já está cadastrada no Save Center.".into());
        }
        let game = SaveGame { id: unique_id(), name: name.to_string(), save_path: path.to_string_lossy().into_owned(), added_at: stamp() };
        games.push(game.clone());
        write_games(&app, &games)?;
        Ok(game)
    }).await.map_err(|_| "Falha ao cadastrar o jogo.".to_string())?
}

#[tauri::command]
pub async fn remove_save_game(app: AppHandle, game_id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut games = read_games(&app)?;
        let before = games.len();
        games.retain(|g| g.id != game_id);
        if games.len() == before { return Err("Jogo não encontrado.".into()); }
        write_games(&app, &games)
    }).await.map_err(|_| "Falha ao remover o jogo do catálogo.".to_string())?
}

fn copy_tree(source: &Path, destination: &Path, stats: &mut (u64, u64)) -> io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "links simbólicos não são aceitos"));
    }
    if metadata.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()), stats)?;
        }
    } else if metadata.is_file() {
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent)?; }
        fs::copy(source, destination)?;
        stats.0 = stats.0.saturating_add(metadata.len());
        stats.1 = stats.1.saturating_add(1);
    }
    Ok(())
}

fn find_game(games: &[SaveGame], id: &str) -> Result<SaveGame, String> {
    games.iter().find(|g| g.id == id).cloned().ok_or_else(|| "Jogo não encontrado no Save Center.".into())
}

#[tauri::command]
pub async fn create_save_backup(app: AppHandle, game_id: String) -> Result<SaveBackup, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let game = find_game(&read_games(&app)?, &game_id)?;
        let source = PathBuf::from(&game.save_path).canonicalize().map_err(|_| "A pasta de saves não está mais disponível.".to_string())?;
        if !source.is_dir() { return Err("A origem do save não é uma pasta.".into()); }
        let id = unique_id();
        let root = backup_root(&app)?.join(&game.id);
        fs::create_dir_all(&root).map_err(|_| "Não foi possível criar a pasta do jogo.".to_string())?;
        let destination = root.join(&id);
        fs::create_dir_all(&destination).map_err(|_| "Não foi possível iniciar o backup.".to_string())?;
        let mut stats = (0u64, 0u64);
        if let Err(error) = copy_tree(&source, &destination.join("payload"), &mut stats) {
            let _ = fs::remove_dir_all(&destination);
            return Err(format!("Backup cancelado: {}. Nenhum arquivo original foi alterado.", error));
        }
        let backup = SaveBackup {
            id, game_id: game.id, game_name: game.name, created_at: stamp(),
            source_path: source.to_string_lossy().into_owned(), size_bytes: stats.0,
            file_count: stats.1, backup_path: destination.to_string_lossy().into_owned(),
        };
        let manifest = serde_json::to_vec_pretty(&backup).map_err(|_| "Não foi possível preparar os dados do backup.".to_string())?;
        if let Err(error) = fs::write(destination.join("backup.json"), manifest) {
            let _ = fs::remove_dir_all(&destination);
            return Err(format!("Não foi possível finalizar o backup: {error}"));
        }
        Ok(backup)
    }).await.map_err(|_| "Falha ao criar backup em segundo plano.".to_string())?
}

fn list_backups_impl(app: &AppHandle, game_id: Option<String>) -> Result<Vec<SaveBackup>, String> {
    let root = backup_root(app)?;
    let mut backups = Vec::new();
    let games = read_games(app)?;
    let selected: Vec<SaveGame> = games.into_iter().filter(|g| game_id.as_ref().map(|id| id == &g.id).unwrap_or(true)).collect();
    for game in selected {
        let folder = root.join(&game.id);
        let Ok(entries) = fs::read_dir(folder) else { continue };
        for entry in entries.flatten() {
            let manifest = entry.path().join("backup.json");
            let Ok(raw) = fs::read_to_string(manifest) else { continue };
            if let Ok(backup) = serde_json::from_str::<SaveBackup>(&raw) { backups.push(backup); }
        }
    }
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(backups)
}

#[tauri::command]
pub async fn list_save_backups(app: AppHandle, game_id: Option<String>) -> Result<Vec<SaveBackup>, String> {
    tauri::async_runtime::spawn_blocking(move || list_backups_impl(&app, game_id)).await
        .map_err(|_| "Falha ao carregar o histórico de backups.".to_string())?
}

#[tauri::command]
pub async fn restore_save_backup(app: AppHandle, backup_id: String, game_id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let game = find_game(&read_games(&app)?, &game_id)?;
        let backups = list_backups_impl(&app, Some(game_id.clone()))?;
        let backup = backups.into_iter().find(|b| b.id == backup_id).ok_or_else(|| "Backup não encontrado para este jogo.".to_string())?;
        let target = PathBuf::from(&game.save_path).canonicalize().map_err(|_| "A pasta de destino não está disponível.".to_string())?;
        if !target.is_dir() { return Err("O destino atual não é uma pasta.".into()); }
        let backup_dir = backup_root(&app)?.join(&game.id).join(&backup.id);
        let payload = backup_dir.join("payload");
        if !payload.is_dir() { return Err("O conteúdo deste backup está incompleto.".into()); }
        let parent = target.parent().ok_or_else(|| "Não foi possível determinar a pasta pai do save.".to_string())?;
        let stage = parent.join(format!(".diamandis-restore-{}", stamp()));
        let safety = parent.join(format!(".diamandis-pre-restore-{}", stamp()));
        let mut stats = (0u64, 0u64);
        copy_tree(&payload, &stage, &mut stats).map_err(|e| { let _ = fs::remove_dir_all(&stage); format!("Não foi possível preparar a restauração: {e}") })?;
        if stats.0 != backup.size_bytes || stats.1 != backup.file_count {
            let _ = fs::remove_dir_all(&stage);
            return Err("A verificação do backup falhou. O save atual permaneceu intacto.".into());
        }
        fs::rename(&target, &safety).map_err(|e| { let _ = fs::remove_dir_all(&stage); format!("Não foi possível proteger o save atual antes da restauração: {e}") })?;
        if let Err(error) = fs::rename(&stage, &target) {
            let _ = fs::rename(&safety, &target);
            let _ = fs::remove_dir_all(&stage);
            return Err(format!("Restauração não concluída. Tentamos devolver o save original ao local: {error}"));
        }
        Ok(format!("Save restaurado. O estado anterior foi preservado em: {}", safety.to_string_lossy()))
    }).await.map_err(|_| "Falha ao restaurar backup em segundo plano.".to_string())?
}

#[cfg(test)]
mod tests {
    use super::copy_tree;
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    #[test]
    fn copies_nested_save_files_and_counts_bytes() {
        let root = std::env::temp_dir().join(format!("diamandis-save-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let source = root.join("source");
        let target = root.join("target");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/save.dat"), b"save-data").unwrap();
        let mut stats = (0, 0);
        copy_tree(&source, &target, &mut stats).unwrap();
        assert_eq!(fs::read(target.join("nested/save.dat")).unwrap(), b"save-data");
        assert_eq!(stats, (9, 1));
        fs::remove_dir_all(root).unwrap();
    }
}

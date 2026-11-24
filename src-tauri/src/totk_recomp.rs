use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::{Path, PathBuf}, process::Command, sync::{Mutex, MutexGuard}, time::{SystemTime, UNIX_EPOCH}};
use tauri::{AppHandle, Manager};

static TOTK_OPERATION_LOCK: Mutex<()> = Mutex::new(());

pub fn operation_lock() -> Result<MutexGuard<'static, ()>, String> {
    TOTK_OPERATION_LOCK.lock().map_err(|_| "O controle de segurança do TOTK foi interrompido. Reinicie o aplicativo antes de tentar novamente.".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotkCampaign {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TotkConfig {
    install_path: String,
    campaigns: Vec<TotkCampaign>,
}

fn stamp() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().to_string()
}

fn unique_id() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos().to_string()
}
fn root(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app.path().app_local_data_dir().map_err(|_| "Não foi possível localizar os dados locais.".to_string())?.join("totk-recomp");
    fs::create_dir_all(&path).map_err(|_| "Não foi possível preparar os dados do adaptador TOTK.".to_string())?;
    Ok(path)
}
fn config_file(app: &AppHandle) -> Result<PathBuf, String> { Ok(root(app)?.join("config.json")) }
fn read_config(app: &AppHandle) -> Result<TotkConfig, String> {
    let path = config_file(app)?;
    if !path.exists() { return Ok(TotkConfig::default()); }
    let raw = fs::read_to_string(path).map_err(|_| "Não foi possível ler as configurações do TOTK Recomp.".to_string())?;
    serde_json::from_str(&raw).map_err(|_| "As configurações do TOTK Recomp estão inválidas.".to_string())
}
fn write_config(app: &AppHandle, config: &TotkConfig) -> Result<(), String> {
    let path = config_file(app)?;
    let temp = path.with_file_name(format!("config-{}.tmp", unique_id()));
    let bytes = serde_json::to_vec_pretty(config)
        .map_err(|_| "Não foi possível preparar as configurações.".to_string())?;
    let result = (|| -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        crate::atomic_file::replace(&temp, &path)?;
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&temp); }
    result.map_err(|_| "Não foi possível concluir a gravação segura das configurações.".to_string())
}
fn validate_install(path: &Path) -> Result<PathBuf, String> {
    let install = path.canonicalize().map_err(|_| "A pasta selecionada não existe ou não pode ser acessada.".to_string())?;
    if !install.is_dir() { return Err("Selecione a pasta que contém totk.exe.".into()); }
    let exe = install.join("totk.exe");
    if !exe.is_file() { return Err("totk.exe não foi encontrado nesta pasta. Selecione a pasta correta da instalação.".into()); }
    Ok(install)
}
fn safe_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    let stem = name.split('.').next().unwrap_or("").trim().to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" | "COM¹" | "COM²" | "COM³" | "LPT¹" | "LPT²" | "LPT³")
        || ["COM", "LPT"].iter().any(|prefix| stem.strip_prefix(*prefix)
            .and_then(|suffix| suffix.parse::<u8>().ok())
            .is_some_and(|number| (1..=9).contains(&number)));
    if name.is_empty() || name.len() > 80 || reserved
        || name.chars().any(|c| c.is_control() || ['<','>',':','"','/','\\','|','?','*'].contains(&c))
        || name.ends_with('.') || name.ends_with(' ') {
        return Err("Use um nome de campanha válido para o Windows, com até 80 caracteres e sem nomes reservados.".into());
    }
    Ok(name.to_string())
}
fn copy_profile(source: &Path, destination: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source).map_err(|_| "A pasta profile original não foi encontrada na instalação.".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() { return Err("A origem profile não é uma pasta segura.".into()); }
    // Reserve the destination exclusively to avoid merging into a folder
    // that appeared after the existence check.
    fs::create_dir(destination).map_err(|_| "Não foi possível reservar uma pasta nova para a campanha.".to_string())?;
    fn visit(src: &Path, dst: &Path) -> std::io::Result<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let from = entry.path();
            let to = dst.join(entry.file_name());
            let meta = fs::symlink_metadata(&from)?;
            if meta.file_type().is_symlink() { return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "link simbólico recusado")); }
            if meta.is_dir() { fs::create_dir(&to)?; visit(&from, &to)?; }
            else if meta.is_file() { fs::copy(&from, &to)?; }
        }
        Ok(())
    }
    if let Err(error) = visit(source, destination) {
        let _ = fs::remove_dir_all(destination);
        return Err(format!("A importação foi cancelada e a cópia incompleta removida. A origem original não foi alterada: {error}"));
    }
    Ok(())
}
fn game_running() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("tasklist");
        command.args(["/FI", "IMAGENAME eq totk.exe", "/FO", "CSV", "/NH"]);
        #[cfg(target_os = "windows")]
        { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
        let output = command.output().map_err(|_| "Não foi possível verificar se totk.exe está em execução.".to_string())?;
        if !output.status.success() { return Err("O Windows não confirmou o estado do processo totk.exe.".into()); }
        let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
        return Ok(text.contains("\"totk.exe\""));
    }
    #[cfg(not(target_os = "windows"))]
    { Err("O adaptador TOTK Recomp está disponível somente no Windows.".into()) }
}

#[tauri::command]
pub async fn configure_totk_install(app: AppHandle, install_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation_lock()?;
        if game_running()? { return Err("Feche o TOTK Recomp antes de alterar a instalação configurada.".into()); }
        let install = validate_install(Path::new(&install_path))?;
        let mut config = read_config(&app)?;
        config.install_path = install.to_string_lossy().into_owned();
        write_config(&app, &config)?;
        Ok(config.install_path)
    }).await.map_err(|_| "Falha ao validar a instalação em segundo plano.".to_string())?
}
#[tauri::command]
pub async fn get_totk_install(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(read_config(&app)?.install_path))
        .await.map_err(|_| "Falha ao carregar a instalação configurada.".to_string())?
}
#[tauri::command]
pub async fn list_totk_campaigns(app: AppHandle) -> Result<Vec<TotkCampaign>, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(read_config(&app)?.campaigns)).await.map_err(|_| "Falha ao carregar campanhas.".to_string())?
}
#[tauri::command]
pub async fn import_totk_profile(app: AppHandle, name: String) -> Result<TotkCampaign, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation_lock()?;
        let name = safe_name(&name)?;
        let config = read_config(&app)?;
        let install = validate_install(Path::new(&config.install_path))?;
        if game_running()? { return Err("Feche o TOTK Recomp antes de importar ou alterar campanhas.".into()); }
        let source = install.join("profile");
        if !source.is_dir() { return Err("A pasta profile original não foi encontrada. Nenhuma alteração foi feita.".into()); }
        let id = unique_id();
        let destination = root(&app)?.join("campaigns").join(&id);
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent).map_err(|_| "Não foi possível preparar a biblioteca de campanhas.".to_string())?; }
        copy_profile(&source, &destination)?;
        let campaign = TotkCampaign { id, name, path: destination.to_string_lossy().into_owned(), created_at: stamp() };
        let mut next = config; next.campaigns.push(campaign.clone());
        if let Err(error) = write_config(&app, &next) { let _ = fs::remove_dir_all(&destination); return Err(error); }
        Ok(campaign)
    }).await.map_err(|_| "Falha ao importar campanha em segundo plano.".to_string())?
}
#[tauri::command]
pub async fn create_empty_totk_campaign(app: AppHandle, name: String) -> Result<TotkCampaign, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation_lock()?;
        let name = safe_name(&name)?;
        let config = read_config(&app)?;
        validate_install(Path::new(&config.install_path))?;
        if game_running()? { return Err("Feche o TOTK Recomp antes de criar ou alterar campanhas.".into()); }
        let id = unique_id();
        let destination = root(&app)?.join("campaigns").join(&id);
        fs::create_dir(&destination).map_err(|_| "Não foi possível reservar uma pasta nova para a campanha vazia.".to_string())?;
        let campaign = TotkCampaign { id, name, path: destination.to_string_lossy().into_owned(), created_at: stamp() };
        let mut next = config; next.campaigns.push(campaign.clone());
        if let Err(error) = write_config(&app, &next) { let _ = fs::remove_dir_all(&destination); return Err(error); }
        Ok(campaign)
    }).await.map_err(|_| "Falha ao criar campanha em segundo plano.".to_string())?
}
#[tauri::command]
pub async fn launch_totk_campaign(app: AppHandle, campaign_id: String) -> Result<crate::save_center::SaveBackup, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation_lock()?;
        if game_running()? { return Err("O TOTK Recomp já está em execução.".into()); }
        let config = read_config(&app)?;
        let install = validate_install(Path::new(&config.install_path))?;
        let campaign = config.campaigns.iter().find(|c| c.id == campaign_id).ok_or_else(|| "Campanha não encontrada.".to_string())?;
        let campaign_path = PathBuf::from(&campaign.path).canonicalize().map_err(|_| "A pasta da campanha não está disponível.".to_string())?;
        let managed_root = root(&app)?.join("campaigns").canonicalize().map_err(|_| "A biblioteca de campanhas não está disponível.".to_string())?;
        if !campaign_path.starts_with(&managed_root) || !campaign_path.is_dir() { return Err("O caminho da campanha não pertence à biblioteca segura do DIAMANDIS ONE.".into()); }
        // O backup é obrigatório no backend, não apenas na interface. Assim,
        // nenhuma chamada IPC consegue iniciar uma campanha sem proteção prévia.
        let backup = crate::save_center::create_backup_for_path(&app, &campaign_path)?;
        let mut command = Command::new(install.join("totk.exe"));
        command.current_dir(&install).env("TOTK_PROFILE_DIR", &campaign_path);
        #[cfg(target_os = "windows")]
        { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
        command.spawn().map_err(|_| "O backup foi concluído, mas não foi possível iniciar totk.exe. Verifique a instalação e as permissões.".to_string())?;
        Ok(backup)
    }).await.map_err(|_| "Falha ao iniciar o jogo em segundo plano.".to_string())?
}
#[tauri::command]
pub async fn is_totk_running() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(game_running).await.map_err(|_| "Falha ao consultar o processo do jogo.".to_string())?
}

pub fn ensure_campaign_stopped(app: &AppHandle, save_path: &str) -> Result<(), String> {
    let config = read_config(app)?;
    let candidate = PathBuf::from(save_path).canonicalize().ok();
    // Two missing paths both produce None; they must never compare as the
    // same managed campaign.
    let managed = candidate.as_ref().is_some_and(|candidate| {
        config.campaigns.iter().any(|campaign| {
            PathBuf::from(&campaign.path).canonicalize().ok().as_ref() == Some(candidate)
        })
    });
    if managed && game_running()? {
        return Err("Feche o TOTK Recomp antes de criar ou restaurar backups desta campanha.".into());
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::safe_name;

    #[test]
    fn accepts_normal_campaign_names() {
        assert_eq!(safe_name("Principal 100%").unwrap(), "Principal 100%");
        assert_eq!(safe_name("Campanha - Sem glitches").unwrap(), "Campanha - Sem glitches");
    }

    #[test]
    fn rejects_empty_or_unsafe_campaign_names() {
        for value in ["", "   ", "../fora", "C:\\temp", "nome|invalido", "nome?", "CON", "NUL.txt", "COM1", "LPT9.log"] {
            assert!(safe_name(value).is_err(), "nome inseguro aceito: {value}");
        }
    }

    #[test]
    fn trims_campaign_names_and_enforces_length() {
        assert_eq!(safe_name("  Principal  ").unwrap(), "Principal");
        assert!(safe_name(&"a".repeat(81)).is_err());
    }
}

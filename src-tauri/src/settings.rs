use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub default_folder: String,
    pub confirm_before_delete: bool,
    pub open_last_folder: bool,
    pub last_folder: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            default_folder: String::new(),
            confirm_before_delete: true,
            open_last_folder: false,
            last_folder: String::new(),
        }
    }
}

fn settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir()
        .map_err(|_| "Não foi possível localizar a pasta de configurações do aplicativo.".to_string())?;
    fs::create_dir_all(&directory)
        .map_err(|_| "Não foi possível preparar a pasta de configurações.".to_string())?;
    Ok(directory.join("settings.json"))
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_file(&app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let contents = fs::read_to_string(path)
        .map_err(|_| "Não foi possível ler as configurações. Os padrões continuam disponíveis.".to_string())?;
    let settings: AppSettings = serde_json::from_str(&contents)
        .map_err(|_| "O arquivo de configurações está inválido. Salve as preferências novamente.".to_string())?;
    validate_settings(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    validate_settings(&settings)?;
    let path = settings_file(&app)?;
    let json = serde_json::to_vec_pretty(&settings)
        .map_err(|_| "Não foi possível preparar as configurações.".to_string())?;
    let mut file = fs::File::create(path)
        .map_err(|_| "Não foi possível salvar as configurações no computador.".to_string())?;
    use std::io::Write;
    file.write_all(&json)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Não foi possível concluir a gravação das configurações.".to_string())
}

fn validate_settings(settings: &AppSettings) -> Result<(), String> {
    if !matches!(settings.theme.as_str(), "dark" | "light" | "system") {
        return Err("Tema selecionado inválido.".into());
    }
    if settings.default_folder.len() > 32_000 || settings.default_folder.contains('\0') || settings.last_folder.len() > 32_000 {
        return Err("A pasta inicial informada é inválida.".into());
    }
    Ok(())
}

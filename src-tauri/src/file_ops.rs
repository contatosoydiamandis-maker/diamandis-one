use std::{fs, path::{Component, Path, PathBuf}};

fn valid_name(name: &str) -> Result<&str, String> {
    let clean = name.trim();
    let stem = clean.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| stem.strip_prefix(*prefix)
            .and_then(|suffix| suffix.parse::<u8>().ok())
            .is_some_and(|n| (1..=9).contains(&n)));
    let invalid = clean.chars().any(|c| c.is_control() || matches!(c, '<'|'>'|':'|'"'|'/'|'\\'|'|'|'?'|'*'));
    let candidate = Path::new(clean);
    if clean.is_empty() || clean.ends_with(' ') || clean.ends_with('.') || reserved || invalid
        || candidate.components().count() != 1
        || !matches!(candidate.components().next(), Some(Component::Normal(_))) {
        return Err("Use um nome válido para o Windows, sem caminhos, caracteres especiais ou nomes reservados.".into());
    }
    Ok(clean)
}

fn existing_item(path: &str) -> Result<PathBuf, String> {
    let original = PathBuf::from(path);
    let metadata = fs::symlink_metadata(&original)
        .map_err(|_| "O item não existe ou não pode ser acessado.".to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("Por segurança, esta operação não altera links simbólicos.".into());
    }
    original.canonicalize()
        .map_err(|_| "Não foi possível acessar o item selecionado.".to_string())
}

fn destination_folder(path: &str) -> Result<PathBuf, String> {
    let folder = PathBuf::from(path).canonicalize()
        .map_err(|_| "A pasta de destino não foi encontrada.".to_string())?;
    if !folder.is_dir() {
        return Err("O destino selecionado não é uma pasta.".into());
    }
    Ok(folder)
}

#[tauri::command]
pub fn rename_item(path: String, new_name: String) -> Result<(), String> {
    let name = valid_name(&new_name)?;
    let source = existing_item(&path)?;
    let parent = source.parent().ok_or("Não é possível renomear este local.")?;
    let destination = parent.join(name);
    if destination.exists() {
        return Err("Já existe um arquivo ou pasta com esse nome.".into());
    }
    fs::rename(source, destination)
        .map_err(|_| "O Windows não permitiu renomear este item. Verifique se ele está em uso ou protegido.".into())
}

#[tauri::command]
pub fn copy_file(path: String, destination_folder_path: String) -> Result<(), String> {
    let source = existing_item(&path)?;
    if !source.is_file() {
        return Err("A cópia de pastas ainda não está disponível. Selecione um arquivo.".into());
    }
    let folder = destination_folder(&destination_folder_path)?;
    let name = source.file_name().ok_or("Nome de arquivo inválido.")?;
    let destination = folder.join(name);
    if destination.exists() {
        return Err("Já existe um item com esse nome no destino. Nada foi sobrescrito.".into());
    }
    let mut input = fs::File::open(&source).map_err(|_| "Não foi possível abrir o arquivo de origem.".to_string())?;
    let mut output = fs::OpenOptions::new().write(true).create_new(true).open(&destination)
        .map_err(|_| "Não foi possível criar a cópia no destino. Verifique as permissões e o espaço disponível.".to_string())?;
    if let Err(_) = std::io::copy(&mut input, &mut output) {
        let _ = fs::remove_file(&destination);
        return Err("A cópia foi interrompida. O arquivo incompleto foi removido.".into());
    }
    Ok(())
}

#[tauri::command]
pub fn move_item(path: String, destination_folder_path: String) -> Result<(), String> {
    let source = existing_item(&path)?;
    let folder = destination_folder(&destination_folder_path)?;
    let name = source.file_name().ok_or("Nome de item inválido.")?;
    let destination = folder.join(name);
    if destination.exists() {
        return Err("Já existe um item com esse nome no destino. Nada foi sobrescrito.".into());
    }
    fs::rename(source, destination)
        .map_err(|_| "Não foi possível mover o item. Verifique se origem e destino estão no mesmo disco e se o item não está em uso.".into())
}

#[tauri::command]
pub fn delete_item(path: String) -> Result<(), String> {
    let target = existing_item(&path)?;
    if target.parent().is_none() {
        return Err("Por segurança, não é permitido enviar a raiz de um disco para a Lixeira.".into());
    }
    trash::delete(&target)
        .map_err(|_| "O Windows não conseguiu enviar este item para a Lixeira. Nada foi excluído permanentemente.".into())
}

use std::{fs, io, path::{Component, Path, PathBuf}};

pub fn validate_name(name: &str) -> Result<&str, String> {
    let clean = name.trim();
    let stem = clean.split('.').next().unwrap_or("").trim().to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" | "COM¹" | "COM²" | "COM³" | "LPT¹" | "LPT²" | "LPT³")
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
pub async fn rename_item(path: String, new_name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || rename_item_impl(path, new_name)).await
        .map_err(|_| "Não foi possível concluir a operação em segundo plano.".to_string())?
}

fn rename_item_impl(path: String, new_name: String) -> Result<(), String> {
    let name = validate_name(&new_name)?;
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
pub async fn copy_file(path: String, destination_folder_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || copy_file_impl(path, destination_folder_path)).await
        .map_err(|_| "Não foi possível concluir a cópia em segundo plano.".to_string())?
}

fn copy_file_impl(path: String, destination_folder_path: String) -> Result<(), String> {
    let source = existing_item(&path)?;
    let folder = destination_folder(&destination_folder_path)?;
    let name = source.file_name().ok_or("Nome de arquivo inválido.")?;
    let destination = folder.join(name);
    if destination.exists() {
        return Err("Já existe um item com esse nome no destino. Nada foi sobrescrito.".into());
    }
    if folder.starts_with(&source) {
        return Err("Não é possível copiar uma pasta para dentro dela mesma ou de uma subpasta.".into());
    }
    let result = if source.is_dir() {
        copy_directory_new(&source, &destination)
    } else if source.is_file() {
        copy_file_new(&source, &destination)
    } else {
        return Err("Este tipo de item não pode ser copiado.".into());
    };
    result.map_err(|_| "Não foi possível concluir a cópia. Verifique as permissões e o espaço disponível; a origem foi preservada e a cópia incompleta foi removida.".into())
}

fn copy_file_new(source: &Path, destination: &Path) -> io::Result<()> {
    let mut input = fs::File::open(source)?;
    let mut output = fs::OpenOptions::new().write(true).create_new(true).open(destination)?;
    let result = (|| {
        io::copy(&mut input, &mut output)?;
        output.sync_all()?;
        let source_len = input.metadata()?.len();
        let destination_len = output.metadata()?.len();
        if source_len != destination_len {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "file size changed during copy"));
        }
        Ok(())
    })();
    if result.is_err() {
        drop(output);
        let _ = fs::remove_file(destination);
    }
    result
}

fn copy_directory_new(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir(destination)?;
    let result = (|| {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "symbolic links are not copied"));
            }
            if kind.is_dir() {
                copy_directory_new(&source_path, &destination_path)?;
            } else if kind.is_file() {
                copy_file_new(&source_path, &destination_path)?;
            } else {
                return Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported filesystem entry"));
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(destination);
    }
    result
}

fn is_cross_device_error(error: &io::Error) -> bool {
    #[cfg(windows)]
    { error.raw_os_error() == Some(17) }
    #[cfg(unix)]
    { error.raw_os_error() == Some(18) }
    #[cfg(not(any(windows, unix)))]
    { let _ = error; false }
}

fn remove_moved_source(source: &Path) -> io::Result<()> {
    if source.is_dir() {
        fs::remove_dir_all(source)
    } else {
        fs::remove_file(source)
    }
}

#[tauri::command]
pub async fn move_item(path: String, destination_folder_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || move_item_impl(path, destination_folder_path)).await
        .map_err(|_| "Não foi possível concluir a movimentação em segundo plano.".to_string())?
}

fn move_item_impl(path: String, destination_folder_path: String) -> Result<(), String> {
    let source = existing_item(&path)?;
    let folder = destination_folder(&destination_folder_path)?;
    let name = source.file_name().ok_or("Nome de item inválido.")?;
    let destination = folder.join(name);
    if destination.exists() {
        return Err("Já existe um item com esse nome no destino. Nada foi sobrescrito.".into());
    }
    if folder.starts_with(&source) {
        return Err("Não é possível mover uma pasta para dentro dela mesma ou de uma subpasta.".into());
    }

    match fs::rename(&source, &destination) {
        Ok(()) => Ok(()),
        Err(error) if is_cross_device_error(&error) => {
            let copy_result = if source.is_dir() {
                copy_directory_new(&source, &destination)
            } else if source.is_file() {
                copy_file_new(&source, &destination)
            } else {
                return Err("Este tipo de item não pode ser movido entre unidades.".into());
            };
            copy_result.map_err(|_| "A cópia entre unidades falhou. A origem foi preservada e qualquer cópia incompleta foi removida.".to_string())?;
            remove_moved_source(&source).map_err(|_| {
                "A cópia foi concluída, mas o Windows não permitiu remover a origem. Os arquivos foram preservados nos dois locais; confira antes de apagar qualquer cópia.".to_string()
            })
        }
        Err(_) => Err("Não foi possível mover o item. Verifique permissões, espaço disponível e se o item está em uso.".into()),
    }
}

#[tauri::command]
pub async fn delete_item(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || delete_item_impl(path)).await
        .map_err(|_| "Não foi possível enviar o item à Lixeira em segundo plano.".to_string())?
}

fn delete_item_impl(path: String) -> Result<(), String> {
    let target = existing_item(&path)?;
    if target.parent().is_none() {
        return Err("Por segurança, não é permitido enviar a raiz de um disco para a Lixeira.".into());
    }
    trash::delete(&target)
        .map_err(|_| "O Windows não conseguiu enviar este item para a Lixeira. Nada foi excluído permanentemente.".into())
}

#[cfg(test)]
mod tests {
    use super::{copy_directory_new, copy_file_impl, copy_file_new, move_item_impl, validate_name};
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    fn temp_root(label: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "diamandis-file-ops-{label}-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn accepts_normal_windows_names() {
        assert_eq!(validate_name("My saves").unwrap(), "My saves");
        assert_eq!(validate_name("Save_2026").unwrap(), "Save_2026");
    }

    #[test]
    fn rejects_paths_reserved_names_and_invalid_characters() {
        for name in ["", " ", "..", "../outside", "folder\\\\child", "CON", "NUL.txt", "COM1", "LPT9.log", "NUL .txt", "CONOUT$", "COM¹", "bad?.txt", "trailing."] {
            assert!(validate_name(name).is_err(), "name should be rejected: {name}");
        }
    }

    #[test]
    fn refuses_to_overwrite_an_existing_file_during_copy() {
        let root = temp_root("no-overwrite");
        let source = root.join("source.txt");
        let destination = root.join("destination.txt");
        fs::write(&source, b"source").unwrap();
        fs::write(&destination, b"keep me").unwrap();

        assert!(copy_file_new(&source, &destination).is_err());
        assert_eq!(fs::read(&destination).unwrap(), b"keep me");
        assert_eq!(fs::read(&source).unwrap(), b"source");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn copies_a_folder_from_the_file_manager_and_preserves_the_source() {
        let root = temp_root("copy-folder");
        let source = root.join("source");
        let destination_root = root.join("destination-root");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::create_dir_all(&destination_root).unwrap();
        fs::write(source.join("nested/save.dat"), b"game-save").unwrap();

        copy_file_impl(
            source.to_string_lossy().into_owned(),
            destination_root.to_string_lossy().into_owned(),
        ).unwrap();

        assert_eq!(fs::read(destination_root.join("source/nested/save.dat")).unwrap(), b"game-save");
        assert!(source.join("nested/save.dat").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_copy_a_folder_into_itself() {
        let root = temp_root("copy-descendant");
        let source = root.join("source");
        fs::create_dir_all(source.join("nested")).unwrap();
        let result = copy_file_impl(
            source.to_string_lossy().into_owned(),
            source.join("nested").to_string_lossy().into_owned(),
        );
        assert!(result.is_err());
        assert!(source.join("nested").is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn copies_directory_trees_without_touching_the_source() {
        let root = temp_root("copy-tree");
        let source = root.join("source");
        let destination = root.join("destination");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("root.txt"), b"root data").unwrap();
        fs::write(source.join("nested/child.txt"), b"nested data").unwrap();

        copy_directory_new(&source, &destination).unwrap();

        assert_eq!(fs::read(destination.join("root.txt")).unwrap(), b"root data");
        assert_eq!(fs::read(destination.join("nested/child.txt")).unwrap(), b"nested data");
        assert!(source.join("root.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symbolic_links_in_directory_copies() {
        use std::os::unix::fs::symlink;
        let root = temp_root("symlink");
        let source = root.join("source");
        let destination = root.join("destination");
        fs::create_dir(&source).unwrap();
        fs::write(root.join("outside.txt"), b"outside").unwrap();
        symlink(root.join("outside.txt"), source.join("link.txt")).unwrap();

        assert!(copy_directory_new(&source, &destination).is_err());
        assert!(!destination.exists());
        assert_eq!(fs::read(root.join("outside.txt")).unwrap(), b"outside");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_moving_a_directory_into_its_descendant() {
        let root = temp_root("descendant");
        let source = root.join("source");
        fs::create_dir_all(source.join("nested")).unwrap();
        let result = move_item_impl(source.to_string_lossy().into_owned(), source.join("nested").to_string_lossy().into_owned());
        assert!(result.is_err());
        assert!(source.join("nested").is_dir());
        fs::remove_dir_all(root).unwrap();
    }
}

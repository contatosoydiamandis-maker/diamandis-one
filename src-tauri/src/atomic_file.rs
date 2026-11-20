use std::{io, path::Path};
#[cfg(not(windows))]
use std::fs;

#[cfg(windows)]
pub fn replace(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "Kernel32")]
    extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x8;
    let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination_wide: Vec<u16> = destination.as_os_str().encode_wide().chain(Some(0)).collect();
    let result = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            destination_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn replace(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(test)]
mod tests {
    use super::replace;
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    #[test]
    fn atomically_replaces_an_existing_file() {
        let root = std::env::temp_dir().join(format!(
            "diamandis-atomic-test-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("settings.tmp");
        let destination = root.join("settings.json");
        fs::write(&source, b"new settings").unwrap();
        fs::write(&destination, b"old settings").unwrap();

        replace(&source, &destination).unwrap();

        assert_eq!(fs::read(&destination).unwrap(), b"new settings");
        assert!(!source.exists());
        fs::remove_dir_all(root).unwrap();
    }
}

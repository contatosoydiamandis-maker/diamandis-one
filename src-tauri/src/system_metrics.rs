use serde::Serialize;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy)]
struct CpuTimes {
    idle: u64,
    kernel: u64,
    user: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageDrive {
    pub letter: String,
    pub name: String,
    pub drive_type: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub capacity_available: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetrics {
    pub cpu_usage_percent: Option<f64>,
    pub logical_cores: Option<usize>,
    pub memory_total_bytes: u64,
    pub memory_available_bytes: u64,
    pub storage_total_bytes: Option<u64>,
    pub storage_free_bytes: Option<u64>,
    pub storage_drives: Vec<StorageDrive>,
}

fn drive_type_label(drive_type: u32) -> Option<&'static str> {
    match drive_type {
        2 => Some("Removível"),
        3 => Some("Disco local"),
        4 => Some("Rede"),
        5 => Some("Unidade óptica"),
        6 => Some("Disco virtual"),
        _ => None,
    }
}

#[cfg(windows)]
fn collect() -> Result<SystemMetrics, String> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct FileTime { low: u32, high: u32 }

    impl FileTime {
        fn value(self) -> u64 { ((self.high as u64) << 32) | self.low as u64 }
    }

    #[repr(C)]
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
        fn GlobalMemoryStatusEx(status: *mut MemoryStatusEx) -> i32;
        fn GetLogicalDrives() -> u32;
        fn GetDriveTypeW(root_path: *const u16) -> u32;
        fn GetVolumeInformationW(
            root_path: *const u16,
            volume_name: *mut u16,
            volume_name_size: u32,
            serial_number: *mut u32,
            max_component_length: *mut u32,
            file_system_flags: *mut u32,
            file_system_name: *mut u16,
            file_system_name_size: u32,
        ) -> i32;
        fn GetDiskFreeSpaceExW(
            directory: *const u16,
            available: *mut u64,
            total: *mut u64,
            free: *mut u64,
        ) -> i32;
    }

    let mut idle = FileTime::default();
    let mut kernel = FileTime::default();
    let mut user = FileTime::default();
    let cpu_usage_percent = unsafe {
        if GetSystemTimes(&mut idle, &mut kernel, &mut user) == 0 {
            None
        } else {
            let current = CpuTimes { idle: idle.value(), kernel: kernel.value(), user: user.value() };
            let previous = CPU_TIMES.get_or_init(|| Mutex::new(None));
            match previous.lock() {
                Ok(mut previous) => {
                    let usage = match *previous { Some(old) => calculate_cpu_usage(old, current), None => None };
                    *previous = Some(current);
                    usage
                }
                Err(_) => None,
            }
        }
    };

    let mut memory = MemoryStatusEx {
        length: std::mem::size_of::<MemoryStatusEx>() as u32,
        memory_load: 0,
        total_phys: 0,
        avail_phys: 0,
        total_page_file: 0,
        avail_page_file: 0,
        total_virtual: 0,
        avail_virtual: 0,
        avail_extended_virtual: 0,
    };
    if unsafe { GlobalMemoryStatusEx(&mut memory) } == 0 || memory.total_phys == 0 {
        return Err("O Windows não conseguiu consultar a memória física.".into());
    }

    // Enumerate every mounted drive letter. A volume must remain visible in
    // the File Manager even when its capacity cannot currently be queried
    // (for example, an empty optical drive or an unavailable network share).
    let logical_drives = unsafe { GetLogicalDrives() };
    let mut storage_drives = Vec::new();

    for index in 0..26 {
        if logical_drives & (1 << index) == 0 { continue; }
        let letter = format!("{}:", (b'A' + index as u8) as char);
        let root = format!("{}\\", letter);
        let wide: Vec<u16> = OsStr::new(&root).encode_wide().chain(Some(0)).collect();
        let drive_type = unsafe { GetDriveTypeW(wide.as_ptr()) };
        let Some(drive_type_name) = drive_type_label(drive_type) else { continue; };

        let mut available = 0u64;
        let mut total = 0u64;
        let mut free = 0u64;
        let capacity_available = unsafe {
            GetDiskFreeSpaceExW(wide.as_ptr(), &mut available, &mut total, &mut free)
        } != 0;
        if !capacity_available {
            total = 0;
            free = 0;
        }

        let mut volume_name = [0u16; 261];
        let has_label = unsafe {
            GetVolumeInformationW(
                wide.as_ptr(),
                volume_name.as_mut_ptr(),
                volume_name.len() as u32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
        } != 0;
        let label = if has_label {
            String::from_utf16_lossy(&volume_name[..volume_name.iter().position(|c| *c == 0).unwrap_or(volume_name.len())]).trim().to_string()
        } else {
            String::new()
        };
        let name = if label.is_empty() { letter.clone() } else { label };
        storage_drives.push(StorageDrive {
            letter,
            name,
            drive_type: drive_type_name.into(),
            total_bytes: total,
            free_bytes: free,
            capacity_available,
        });
    }

    storage_drives.sort_by(|a, b| a.letter.cmp(&b.letter));
    let has_capacity_data = storage_drives.iter().any(|drive| drive.capacity_available);
    let (storage_total_bytes, storage_free_bytes) = if !has_capacity_data {
        (None, None)
    } else {
        (
            storage_drives.iter().filter(|drive| drive.capacity_available)
                .try_fold(0u64, |sum, drive| sum.checked_add(drive.total_bytes)),
            storage_drives.iter().filter(|drive| drive.capacity_available)
                .try_fold(0u64, |sum, drive| sum.checked_add(drive.free_bytes)),
        )
    };

    Ok(SystemMetrics {
        cpu_usage_percent,
        logical_cores: std::thread::available_parallelism().ok().map(|n| n.get()),
        memory_total_bytes: memory.total_phys,
        memory_available_bytes: memory.avail_phys,
        storage_total_bytes,
        storage_free_bytes,
        storage_drives,
    })
}

#[cfg(not(windows))]
fn collect() -> Result<SystemMetrics, String> {
    Err("O monitoramento nativo está disponível inicialmente no Windows.".into())
}

fn calculate_cpu_usage(previous: CpuTimes, current: CpuTimes) -> Option<f64> {
    let idle = current.idle.checked_sub(previous.idle)?;
    let kernel = current.kernel.checked_sub(previous.kernel)?;
    let user = current.user.checked_sub(previous.user)?;
    let total = kernel.checked_add(user)?;
    if total == 0 || idle > total { return None; }
    Some(((total - idle) as f64 / total as f64 * 100.0).clamp(0.0, 100.0))
}

static CPU_TIMES: OnceLock<Mutex<Option<CpuTimes>>> = OnceLock::new();

#[tauri::command]
pub async fn get_system_metrics() -> Result<SystemMetrics, String> {
    tauri::async_runtime::spawn_blocking(collect).await
        .map_err(|_| "Não foi possível consultar o sistema em segundo plano.".to_string())?
}

#[cfg(test)]
mod tests {
    use super::{calculate_cpu_usage, drive_type_label, CpuTimes};

    #[test]
    fn classifies_supported_windows_drive_types() {
        assert_eq!(drive_type_label(2), Some("Removível"));
        assert_eq!(drive_type_label(3), Some("Disco local"));
        assert_eq!(drive_type_label(4), Some("Rede"));
        assert_eq!(drive_type_label(5), Some("Unidade óptica"));
        assert_eq!(drive_type_label(6), Some("Disco virtual"));
        assert_eq!(drive_type_label(0), None);
        assert_eq!(drive_type_label(1), None);
    }

    #[test]
    fn calculates_cpu_usage_from_system_time_deltas() {
        let previous = CpuTimes { idle: 100, kernel: 200, user: 100 };
        let current = CpuTimes { idle: 200, kernel: 300, user: 200 };
        assert_eq!(calculate_cpu_usage(previous, current), Some(50.0));
    }

    #[test]
    fn returns_none_when_no_cpu_time_elapsed() {
        let sample = CpuTimes { idle: 10, kernel: 20, user: 30 };
        assert_eq!(calculate_cpu_usage(sample, sample), None);
    }

    #[test]
    fn rejects_counter_regressions() {
        let previous = CpuTimes { idle: 100, kernel: 200, user: 100 };
        let current = CpuTimes { idle: 90, kernel: 220, user: 110 };
        assert_eq!(calculate_cpu_usage(previous, current), None);
    }
}

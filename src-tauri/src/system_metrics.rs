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
pub struct SystemMetrics {
    pub cpu_usage_percent: Option<f64>,
    pub logical_cores: usize,
    pub memory_total_bytes: u64,
    pub memory_available_bytes: u64,
    pub storage_total_bytes: Option<u64>,
    pub storage_free_bytes: Option<u64>,
}

#[cfg(windows)]
fn collect() -> Result<SystemMetrics, String> {
    use std::{env, ffi::OsStr, os::windows::ffi::OsStrExt, path::PathBuf};

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
                    let usage = previous.map(|old| calculate_cpu_usage(old, current));
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

    let drive = env::var_os("SystemDrive").unwrap_or_else(|| OsStr::new("C:").to_os_string());
    let mut root = PathBuf::from(drive);
    root.push("\\");
    let wide: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut available = 0u64;
    let mut total = 0u64;
    let mut free = 0u64;
    let disk_ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut available, &mut total, &mut free) } != 0;

    Ok(SystemMetrics {
        cpu_usage_percent,
        logical_cores: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
        memory_total_bytes: memory.total_phys,
        memory_available_bytes: memory.avail_phys,
        storage_total_bytes: if disk_ok { Some(total) } else { None },
        storage_free_bytes: if disk_ok { Some(free) } else { None },
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
    use super::{calculate_cpu_usage, CpuTimes};

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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MachinePerformance {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub disk_percent: f64,
    pub disk_free_gb: f64,
    pub disk_total_gb: f64,
    pub uptime_days: f64,
}

pub async fn get_local_performance() -> Result<MachinePerformance, String> {
    tokio::task::spawn_blocking(sample)
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[cfg(windows)]
fn sample() -> Result<MachinePerformance, String> {
    native::sample()
}

#[cfg(not(windows))]
fn sample() -> Result<MachinePerformance, String> {
    Err("Performance metrics are only available on Windows".to_string())
}

#[cfg(windows)]
mod native {
    use std::sync::Mutex;
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    use windows_sys::Win32::System::SystemInformation::{
        GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX,
    };
    use windows_sys::Win32::System::Threading::GetSystemTimes;

    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

    #[derive(Clone, Copy)]
    struct CpuTimes {
        idle: u64,
        kernel: u64,
        user: u64,
    }

    static PREVIOUS_CPU: Mutex<Option<CpuTimes>> = Mutex::new(None);

    fn to_u64(time: FILETIME) -> u64 {
        ((time.dwHighDateTime as u64) << 32) | time.dwLowDateTime as u64
    }

    fn cpu_percent() -> f64 {
        let mut idle = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut kernel = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut user = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
            return 0.0;
        }
        let now = CpuTimes {
            idle: to_u64(idle),
            kernel: to_u64(kernel),
            user: to_u64(user),
        };
        let mut previous = match PREVIOUS_CPU.lock() {
            Ok(guard) => guard,
            Err(_) => return 0.0,
        };
        let Some(prev) = *previous else {
            *previous = Some(now);
            return 0.0;
        };
        *previous = Some(now);
        let total_delta =
            now.kernel.saturating_sub(prev.kernel) + now.user.saturating_sub(prev.user);
        if total_delta == 0 {
            return 0.0;
        }
        let busy_delta = total_delta.saturating_sub(now.idle.saturating_sub(prev.idle));
        (busy_delta as f64 / total_delta as f64) * 100.0
    }

    fn memory() -> Result<(f64, f64, f64), String> {
        let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
            return Err("GlobalMemoryStatusEx failed".to_string());
        }
        let total_gb = status.ullTotalPhys as f64 / GIB;
        let free_gb = status.ullAvailPhys as f64 / GIB;
        let used_gb = total_gb - free_gb;
        let percent = if total_gb > 0.0 {
            (used_gb / total_gb) * 100.0
        } else {
            0.0
        };
        Ok((percent, used_gb, total_gb))
    }

    fn disk() -> Result<(f64, f64, f64), String> {
        let root = crate::paths::xampp_root();
        let drive = root
            .to_string_lossy()
            .chars()
            .next()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| format!("{}:\\", c))
            .unwrap_or_else(|| "C:\\".to_string());
        let wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
        let mut free_available = 0u64;
        let mut total = 0u64;
        let mut free_total = 0u64;
        let ok = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_available,
                &mut total,
                &mut free_total,
            )
        };
        if ok == 0 {
            return Err("GetDiskFreeSpaceExW failed".to_string());
        }
        if total == 0 {
            return Err("Disk total is zero".to_string());
        }
        let total_gb = total as f64 / GIB;
        let free_gb = free_total as f64 / GIB;
        let percent = ((total - free_total) as f64 / total as f64) * 100.0;
        Ok((percent, free_gb, total_gb))
    }

    pub fn sample() -> Result<super::MachinePerformance, String> {
        let (memory_percent, memory_used_gb, memory_total_gb) = memory()?;
        let (disk_percent, disk_free_gb, disk_total_gb) = disk()?;
        let uptime_days = unsafe { GetTickCount64() } as f64 / 86_400_000.0;
        let round = |value: f64| (value * 10.0).round() / 10.0;
        Ok(super::MachinePerformance {
            cpu_percent: round(cpu_percent()),
            memory_percent: round(memory_percent),
            memory_used_gb: round(memory_used_gb),
            memory_total_gb: round(memory_total_gb),
            disk_percent: round(disk_percent),
            disk_free_gb: round(disk_free_gb),
            disk_total_gb: round(disk_total_gb),
            uptime_days: round(uptime_days),
        })
    }
}

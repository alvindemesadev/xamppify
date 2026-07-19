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
    let script = r#"
$cpu = (Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
$os = Get-CimInstance Win32_OperatingSystem
$disk = Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" | Where-Object { $_.DeviceID -eq 'C:' }
$memTotal = $os.TotalVisibleMemorySize / 1mb
$memFree = $os.FreePhysicalMemory / 1mb
$memUsed = $memTotal - $memFree
$diskTotal = $disk.Size / 1gb
$diskFree = $disk.FreeSpace / 1gb
$uptime = (Get-Date) - $os.LastBootUpTime
$props = @{
    cpu_percent = [math]::Round($cpu, 1)
    memory_percent = [math]::Round(($memUsed / $memTotal) * 100, 1)
    memory_used_gb = [math]::Round($memUsed, 1)
    memory_total_gb = [math]::Round($memTotal, 1)
    disk_percent = [math]::Round((($diskTotal - $diskFree) / $diskTotal) * 100, 1)
    disk_free_gb = [math]::Round($diskFree, 1)
    disk_total_gb = [math]::Round($diskTotal, 1)
    uptime_days = [math]::Round($uptime.TotalDays, 1)
}
$props | ConvertTo-Json
"#;

    let mut command = tokio::process::Command::new("powershell");
    command
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script);
    crate::paths::hide_console_for_tokio_command(&mut command);

    let output = command
        .output()
        .await
        .map_err(|e| format!("Failed to get performance: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Performance query failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse performance data: {}", e))
}

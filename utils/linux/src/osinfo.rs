// Linux version information structure
#[derive(Default, Debug, Clone)]
pub struct OsVersionInfo {
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    pub platform_id: u32,
    pub service_pack: String,
}

// Gets Linux OS version information
pub fn version_info() -> anyhow::Result<OsVersionInfo> {
    let mut info = OsVersionInfo::default();

    // Get Linux kernel version
    if let Ok(release) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        let parts: Vec<&str> = release.trim().split('.').collect();
        if parts.len() >= 3 {
            if let Ok(major) = parts[0].parse::<u32>() {
                info.major_version = major;
            }
            if let Ok(minor) = parts[1].parse::<u32>() {
                info.minor_version = minor;
            }
            if let Ok(build) = parts[2].parse::<u32>() {
                info.build_number = build;
            }
        }
    }

    // Get distribution info if available
    if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME=") {
                info.service_pack = line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
                break;
            }
        }
    }

    Ok(info)
}

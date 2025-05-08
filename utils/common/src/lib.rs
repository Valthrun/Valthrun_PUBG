// Cross-platform version information structure
#[derive(Default, Debug, Clone)]
pub struct OsVersionInfo {
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    pub platform_id: u32,
    pub service_pack: String,
    pub is_windows: bool,
}

// Cross-platform version info function
pub fn get_os_info() -> anyhow::Result<OsVersionInfo> {
    #[cfg(feature = "windows")]
    {
        use utils_windows::version_info;
        let win_info = version_info()?;
        Ok(OsVersionInfo {
            major_version: win_info.dwMajorVersion,
            minor_version: win_info.dwMinorVersion,
            build_number: win_info.dwBuildNumber,
            platform_id: win_info.dwPlatformId,
            service_pack: String::from_utf16_lossy(&win_info.szCSDVersion)
                .trim_end_matches('\0')
                .to_string(),
            is_windows: true,
        })
    }

    #[cfg(all(feature = "linux", not(feature = "windows")))]
    {
        use utils_linux::version_info;
        let linux_info = version_info()?;
        Ok(OsVersionInfo {
            major_version: linux_info.major_version,
            minor_version: linux_info.minor_version,
            build_number: linux_info.build_number,
            platform_id: linux_info.platform_id,
            service_pack: linux_info.service_pack,
            is_windows: false,
        })
    }

    #[cfg(not(any(feature = "windows", feature = "linux")))]
    {
        anyhow::bail!("No platform-specific utils enabled")
    }
}

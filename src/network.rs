#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub fn tune_tcp_latency() -> Result<usize, String> {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let interfaces_path = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces";
        let root_key = hklm
            .open_subkey_with_flags(interfaces_path, KEY_READ)
            .map_err(|e| e.to_string())?;

        let mut modified = 0;
        for sub_name in root_key.enum_keys().flatten() {
            let sub_path = format!(r"{}\{}", interfaces_path, sub_name);
            if let Ok(if_key) = hklm.open_subkey_with_flags(&sub_path, KEY_SET_VALUE) {
                let _ = if_key.set_value("TcpAckFrequency", &1u32);
                let _ = if_key.set_value("TCPNoDelay", &1u32);
                modified += 1;
            }
        }
        Ok(modified)
    }
    #[cfg(not(windows))]
    {
        Ok(1)
    }
}

pub fn read_mmcss_state() -> (Option<u32>, Option<u32>) {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let profile_path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
        if let Ok(key) = hklm.open_subkey_with_flags(profile_path, KEY_READ) {
            let sys_resp = key.get_value::<u32, _>("SystemResponsiveness").ok();
            let net_thr = key.get_value::<u32, _>("NetworkThrottlingIndex").ok();
            return (sys_resp, net_thr);
        }
    }
    (None, None)
}

pub fn tune_mmcss() -> Result<(), String> {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let profile_path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
        let (key, _) = hklm
            .create_subkey_with_flags(profile_path, KEY_SET_VALUE)
            .map_err(|e| e.to_string())?;

        key.set_value("SystemResponsiveness", &0u32)
            .map_err(|e| e.to_string())?;
        key.set_value("NetworkThrottlingIndex", &0xffffffffu32)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}

pub fn restore_mmcss_state(sys_resp: Option<u32>, net_throttle: Option<u32>) {
    #[cfg(not(windows))]
    {
        let _ = (sys_resp, net_throttle);
    }
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let profile_path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
        if let Ok((key, _)) = hklm.create_subkey_with_flags(profile_path, KEY_SET_VALUE) {
            match sys_resp {
                Some(val) => { let _ = key.set_value("SystemResponsiveness", &val); }
                None => { let _ = key.delete_value("SystemResponsiveness"); }
            }
            match net_throttle {
                Some(val) => { let _ = key.set_value("NetworkThrottlingIndex", &val); }
                None => { let _ = key.delete_value("NetworkThrottlingIndex"); }
            }
        }
    }
}

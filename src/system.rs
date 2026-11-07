use std::path::Path;
use std::process::Command;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub fn flush_dns() -> bool {
    let output = Command::new("ipconfig").arg("/flushdns").output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

pub fn get_active_power_plan_guid() -> Option<String> {
    let out = Command::new("powercfg").arg("/getactivescheme").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    for token in text.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_ascii_hexdigit() && c != '-');
        if clean.len() == 36 && clean.chars().filter(|&c| c == '-').count() == 4 {
            return Some(clean.to_string());
        }
    }
    None
}

pub fn set_power_plan_guid(guid: &str) -> bool {
    let res = Command::new("powercfg")
        .args(["/setactive", guid])
        .output();
    matches!(res, Ok(out) if out.status.success())
}

pub fn set_high_performance_power_plan() -> bool {
    let guid = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
    if set_power_plan_guid(guid) {
        return true;
    }

    let _ = Command::new("powercfg")
        .args(["-duplicatescheme", guid])
        .output();
    set_power_plan_guid(guid)
}

pub fn get_gamebar_state() -> (Option<u32>, Option<u32>) {
    #[cfg(windows)]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let mut app_capture = None;
        let mut game_dvr = None;

        if let Ok(dvr) = hkcu.open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR",
            KEY_READ,
        ) {
            app_capture = dvr.get_value::<u32, _>("AppCaptureEnabled").ok();
        }

        if let Ok(store) = hkcu.open_subkey_with_flags(r"System\GameConfigStore", KEY_READ) {
            game_dvr = store.get_value::<u32, _>("GameDVR_Enabled").ok();
        }

        (app_capture, game_dvr)
    }
    #[cfg(not(windows))]
    {
        (Some(1), Some(1))
    }
}

pub fn disable_xbox_gamebar() -> bool {
    #[cfg(windows)]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let mut ok = false;

        if let Ok((dvr, _)) = hkcu.create_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR",
            KEY_SET_VALUE,
        ) {
            let _ = dvr.set_value("AppCaptureEnabled", &0u32);
            ok = true;
        }

        if let Ok((store, _)) =
            hkcu.create_subkey_with_flags(r"System\GameConfigStore", KEY_SET_VALUE)
        {
            let _ = store.set_value("GameDVR_Enabled", &0u32);
            ok = true;
        }

        ok
    }
    #[cfg(not(windows))]
    {
        true
    }
}

pub fn restore_gamebar_state(app_capture: Option<u32>, game_dvr: Option<u32>) {
    #[cfg(not(windows))]
    {
        let _ = (app_capture, game_dvr);
    }
    #[cfg(windows)]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok((dvr, _)) = hkcu.create_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR",
            KEY_SET_VALUE,
        ) {
            match app_capture {
                Some(v) => { let _ = dvr.set_value("AppCaptureEnabled", &v); }
                None => { let _ = dvr.delete_value("AppCaptureEnabled"); }
            }
        }
        if let Ok((store, _)) =
            hkcu.create_subkey_with_flags(r"System\GameConfigStore", KEY_SET_VALUE)
        {
            match game_dvr {
                Some(v) => { let _ = store.set_value("GameDVR_Enabled", &v); }
                None => { let _ = store.delete_value("GameDVR_Enabled"); }
            }
        }
    }
}

pub fn get_win32_priority_separation() -> Option<u32> {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = r"SYSTEM\CurrentControlSet\Control\PriorityControl";
        if let Ok(key) = hklm.open_subkey_with_flags(path, KEY_READ) {
            key.get_value::<u32, _>("Win32PrioritySeparation").ok()
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        Some(2)
    }
}

pub fn set_win32_priority_separation() -> bool {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = r"SYSTEM\CurrentControlSet\Control\PriorityControl";
        if let Ok((key, _)) = hklm.create_subkey_with_flags(path, KEY_SET_VALUE) {
            key.set_value("Win32PrioritySeparation", &0x26u32).is_ok()
        } else {
            false
        }
    }
    #[cfg(not(windows))]
    {
        true
    }
}

pub fn restore_win32_priority_separation(val: Option<u32>) {
    #[cfg(not(windows))]
    {
        let _ = val;
    }
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = r"SYSTEM\CurrentControlSet\Control\PriorityControl";
        if let Ok((key, _)) = hklm.create_subkey_with_flags(path, KEY_SET_VALUE) {
            match val {
                Some(v) => { let _ = key.set_value("Win32PrioritySeparation", &v); }
                None => { let _ = key.set_value("Win32PrioritySeparation", &2u32); }
            }
        }
    }
}

pub fn get_disable_paging_executive() -> Option<u32> {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management";
        if let Ok(key) = hklm.open_subkey_with_flags(path, KEY_READ) {
            key.get_value::<u32, _>("DisablePagingExecutive").ok()
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        Some(0)
    }
}

pub fn get_total_ram_gb() -> f64 {
    #[cfg(windows)]
    unsafe {
        use crate::winapi_sys::*;
        let mut mem: MEMORYSTATUSEX = std::mem::zeroed();
        mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut mem) != 0 {
            return (mem.ullTotalPhys as f64) / (1024.0 * 1024.0 * 1024.0);
        }
    }
    16.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum PagingExecResult {
    Applied,
    SkippedLowRam(f64),
    Failed,
}

pub fn enable_disable_paging_executive() -> PagingExecResult {
    let ram_gb = get_total_ram_gb();
    if ram_gb < 15.5 {
        return PagingExecResult::SkippedLowRam(ram_gb);
    }

    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management";
        if let Ok((key, _)) = hklm.create_subkey_with_flags(path, KEY_SET_VALUE) {
            if key.set_value("DisablePagingExecutive", &1u32).is_ok() {
                PagingExecResult::Applied
            } else {
                PagingExecResult::Failed
            }
        } else {
            PagingExecResult::Failed
        }
    }
    #[cfg(not(windows))]
    {
        PagingExecResult::Applied
    }
}

pub fn restore_disable_paging_executive(val: Option<u32>) {
    #[cfg(not(windows))]
    {
        let _ = val;
    }
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management";
        if let Ok((key, _)) = hklm.create_subkey_with_flags(path, KEY_SET_VALUE) {
            match val {
                Some(v) => { let _ = key.set_value("DisablePagingExecutive", &v); }
                None => { let _ = key.delete_value("DisablePagingExecutive"); }
            }
        }
    }
}

pub fn get_fullscreen_optimizations_state(cs2_path: &Path) -> Option<String> {
    #[cfg(not(windows))]
    {
        let _ = cs2_path;
        None
    }
    #[cfg(windows)]
    {
        let exe_path = cs2_path.join("game/bin/win64/cs2.exe");
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let layers_path = r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers";
        if let Ok(key) = hkcu.open_subkey_with_flags(layers_path, KEY_READ) {
            let exe_str = exe_path.to_string_lossy();
            return key.get_value::<String, _>(&*exe_str).ok();
        }
        None
    }
}

pub fn disable_fullscreen_optimizations(cs2_path: &Path) -> bool {
    let exe_path = cs2_path.join("game/bin/win64/cs2.exe");
    if !exe_path.exists() {
        return false;
    }

    #[cfg(windows)]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let layers_path = r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers";
        if let Ok((key, _)) = hkcu.create_subkey_with_flags(layers_path, KEY_SET_VALUE) {
            let exe_str = exe_path.to_string_lossy();
            key.set_value(&*exe_str, &"~ DISABLEDXMAXIMIZEDWINDOWEDMODE")
                .is_ok()
        } else {
            false
        }
    }
    #[cfg(not(windows))]
    {
        true
    }
}

pub fn restore_fullscreen_optimizations(cs2_path: &Path, original_val: Option<&str>) {
    #[cfg(not(windows))]
    {
        let _ = (cs2_path, original_val);
    }
    #[cfg(windows)]
    {
        let exe_path = cs2_path.join("game/bin/win64/cs2.exe");
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let layers_path = r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers";
        if let Ok((key, _)) = hkcu.create_subkey_with_flags(layers_path, KEY_SET_VALUE) {
            let exe_str = exe_path.to_string_lossy();
            match original_val {
                Some(val) => { let _ = key.set_value(&*exe_str, &val); }
                None => { let _ = key.delete_value(&*exe_str); }
            }
        }
    }
}

pub fn add_defender_exclusion(cs2_path: &Path) -> bool {
    let path_str = cs2_path.to_string_lossy();
    let script = format!("Add-MpPreference -ExclusionPath '{}'", path_str);
    let res = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();

    matches!(res, Ok(out) if out.status.success())
}

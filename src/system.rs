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

pub fn set_high_performance_power_plan() -> bool {
    let guid = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
    let res = Command::new("powercfg")
        .args(["/setactive", guid])
        .output();

    if let Ok(out) = res {
        if out.status.success() {
            return true;
        }
    }

    let _ = Command::new("powercfg")
        .args(["-duplicatescheme", guid])
        .output();
    let retry = Command::new("powercfg")
        .args(["/setactive", guid])
        .output();
    matches!(retry, Ok(out) if out.status.success())
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

pub fn add_defender_exclusion(cs2_path: &Path) -> bool {
    let path_str = cs2_path.to_string_lossy();
    let script = format!("Add-MpPreference -ExclusionPath '{}'", path_str);
    let res = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();

    matches!(res, Ok(out) if out.status.success())
}

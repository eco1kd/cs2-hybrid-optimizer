use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

#[cfg(windows)]
use crate::winapi_sys::*;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub fn get_steam_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey_with_flags(r"Software\Valve\Steam", KEY_READ) {
            if let Ok(val) = key.get_value::<String, _>("SteamPath") {
                let p = PathBuf::from(val);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        for subkey in [
            r"SOFTWARE\WOW6432Node\Valve\Steam",
            r"SOFTWARE\Valve\Steam",
        ] {
            if let Ok(key) = hklm.open_subkey_with_flags(subkey, KEY_READ) {
                if let Ok(val) = key.get_value::<String, _>("InstallPath") {
                    let p = PathBuf::from(val);
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

pub fn get_cs2_path(steam_path: Option<&Path>) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        for h in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
            let root = RegKey::predef(h);
            for subkey in [
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 730",
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 730",
            ] {
                if let Ok(key) = root.open_subkey_with_flags(subkey, KEY_READ) {
                    if let Ok(val) = key.get_value::<String, _>("InstallLocation") {
                        let p = PathBuf::from(val);
                        if p.exists() {
                            return Some(p);
                        }
                    }
                }
            }
        }
    }

    if let Some(steam) = steam_path {
        let vdf_path = steam.join("steamapps/libraryfolders.vdf");
        if vdf_path.exists() {
            if let Ok(content) = fs::read_to_string(&vdf_path) {
                for line in content.lines() {
                    if line.contains("\"path\"") {
                        let parts: Vec<&str> = line.split('"').collect();
                        if parts.len() >= 4 {
                            let lib_path = parts[3].replace(r"\\", "\\");
                            let candidate = PathBuf::from(lib_path).join("steamapps/common/Counter-Strike 2");
                            if candidate.exists() {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }

        let def = steam.join("steamapps/common/Counter-Strike 2");
        if def.exists() {
            return Some(def);
        }
    }

    None
}

pub fn detect_cpu_cores() -> (usize, usize) {
    let logical = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let mut physical = logical;

    let out = Command::new("wmic")
        .args(["cpu", "get", "NumberOfCores", "/value"])
        .output();
    if let Ok(o) = out {
        let text = String::from_utf8_lossy(&o.stdout);
        for line in text.lines() {
            if let Some(val) = line.strip_prefix("NumberOfCores=") {
                if let Ok(num) = val.trim().parse::<usize>() {
                    physical = num;
                    break;
                }
            }
        }
    }

    (physical.max(1), logical.max(1))
}

pub fn generate_autoexec(cs2_path: &Path) -> Result<PathBuf, String> {
    let cfg_dir = cs2_path.join("game/csgo/cfg");
    fs::create_dir_all(&cfg_dir).map_err(|e| e.to_string())?;

    let autoexec_path = cfg_dir.join("autoexec.cfg");

    if autoexec_path.exists() {
        let now = chrono_timestamp();
        let backup_name = format!("autoexec.backup_{}.cfg", now);
        let backup_path = cfg_dir.join(backup_name);
        let _ = fs::copy(&autoexec_path, backup_path);
    }

    let content = "\
fps_max 0
fps_max_ui 120
engine_low_latency_sleep_after_client_tick true
vprof_off
rate 786432
cl_net_buffer_ticks 0
mm_dedicated_search_maxping 50
r_show_build_info false
voice_modenable 1
snd_voipvolume 0.5
";

    fs::write(&autoexec_path, content).map_err(|e| e.to_string())?;
    Ok(autoexec_path)
}

fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

pub fn launch_cs2(steam_path: Option<&Path>, launch_options: &str) -> bool {
    if let Some(steam) = steam_path {
        let steam_exe = steam.join("steam.exe");
        if steam_exe.exists() {
            let mut cmd = Command::new(&steam_exe);
            cmd.arg("-applaunch").arg("730");
            for opt in launch_options.split_whitespace() {
                cmd.arg(opt);
            }
            if cmd.spawn().is_ok() {
                return true;
            }
        }
    }

    Command::new("cmd")
        .args(["/C", "start", "steam://run/730"])
        .spawn()
        .is_ok()
}

pub fn boost_cs2_process_native() -> bool {
    #[cfg(windows)]
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut pe: PROCESSENTRY32 = std::mem::zeroed();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let mut boosted = false;
        if Process32First(snapshot, &mut pe) != 0 {
            loop {
                let name = std::ffi::CStr::from_ptr(pe.szExeFile.as_ptr())
                    .to_string_lossy();
                if name.eq_ignore_ascii_case("cs2.exe") {
                    let h_proc = OpenProcess(
                        PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION,
                        0,
                        pe.th32ProcessID,
                    );
                    if !h_proc.is_null() {
                        if SetPriorityClass(h_proc, HIGH_PRIORITY_CLASS) != 0 {
                            boosted = true;
                        }
                        CloseHandle(h_proc);
                    }
                }
                if Process32Next(snapshot, &mut pe) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        boosted
    }
    #[cfg(not(windows))]
    {
        true
    }
}

pub fn watch_and_boost_cs2(timeout_secs: u64) -> bool {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if boost_cs2_process_native() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(1500));
    }
    false
}

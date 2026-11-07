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
    #[cfg(not(windows))]
    {
        let _ = &mut physical;
    }

    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = r"HARDWARE\DESCRIPTION\System\CentralProcessor";
        if let Ok(key) = hklm.open_subkey_with_flags(path, KEY_READ) {
            let count = key.enum_keys().count();
            if count > 0 {
                physical = if count > 4 && count % 2 == 0 {
                    count / 2
                } else {
                    count
                };
            }
        }
    }

    (physical.max(1), logical.max(1))
}

pub fn calculate_pcore_affinity_mask(physical_cores: usize, logical_threads: usize) -> Option<usize> {
    if physical_cores >= logical_threads || physical_cores >= usize::BITS as usize {
        return None;
    }
    let mut mask: usize = 0;
    for i in 0..physical_cores {
        mask |= 1 << (i * 2);
    }
    if mask != 0 {
        Some(mask)
    } else {
        None
    }
}

pub fn kill_lingering_cs2_native() -> usize {
    #[cfg(windows)]
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return 0;
        }

        let mut pe: PROCESSENTRY32 = std::mem::zeroed();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let mut killed = 0;
        if Process32First(snapshot, &mut pe) != 0 {
            loop {
                let name = std::ffi::CStr::from_ptr(pe.szExeFile.as_ptr()).to_string_lossy();
                if name.eq_ignore_ascii_case("cs2.exe") {
                    let h_proc = OpenProcess(PROCESS_TERMINATE, 0, pe.th32ProcessID);
                    if !h_proc.is_null() {
                        if TerminateProcess(h_proc, 0) != 0 {
                            killed += 1;
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
        killed
    }
    #[cfg(not(windows))]
    {
        0
    }
}

pub fn generate_autoexec(cs2_path: &Path) -> Result<(PathBuf, Option<PathBuf>), String> {
    let cfg_dir = cs2_path.join("game/csgo/cfg");
    fs::create_dir_all(&cfg_dir).map_err(|e| e.to_string())?;

    let autoexec_path = cfg_dir.join("autoexec.cfg");
    let mut backup_path = None;

    let existing_content = if autoexec_path.exists() {
        let now = chrono_timestamp();
        let b_name = format!("autoexec.backup_{}.cfg", now);
        let b_path = cfg_dir.join(b_name);
        let _ = fs::copy(&autoexec_path, &b_path);
        backup_path = Some(b_path);
        fs::read_to_string(&autoexec_path).unwrap_or_default()
    } else {
        String::new()
    };

    let opt_marker_start = "// === CS2 HYBRID OPTIMIZER START ===";
    let opt_marker_end = "// === CS2 HYBRID OPTIMIZER END ===";

    let opt_block = format!(
        "{}\n\
fps_max 0\n\
fps_max_ui 120\n\
engine_low_latency_sleep_after_client_tick true\n\
vprof_off\n\
rate 1000000\n\
cl_net_buffer_ticks 0\n\
mm_dedicated_search_maxping 50\n\
r_show_build_info false\n\
voice_modenable 1\n\
snd_voipvolume 0.5\n\
{}\n",
        opt_marker_start, opt_marker_end
    );

    let new_content = if existing_content.contains(opt_marker_start) && existing_content.contains(opt_marker_end) {
        let start_idx = existing_content.find(opt_marker_start).unwrap_or(0);
        let end_idx = existing_content.find(opt_marker_end).unwrap_or(existing_content.len()) + opt_marker_end.len();
        let mut merged = existing_content[..start_idx].to_string();
        merged.push_str(&opt_block);
        if end_idx < existing_content.len() {
            merged.push_str(&existing_content[end_idx..]);
        }
        merged
    } else if existing_content.is_empty() {
        opt_block
    } else {
        format!("{}\n\n{}", existing_content.trim_end(), opt_block)
    };

    fs::write(&autoexec_path, new_content).map_err(|e| e.to_string())?;
    Ok((autoexec_path, backup_path))
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

pub fn boost_cs2_process_native(affinity_mask: Option<usize>) -> bool {
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
                        if let Some(mask) = affinity_mask {
                            let _ = SetProcessAffinityMask(h_proc, mask);
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
        let _ = affinity_mask;
        true
    }
}

pub fn watch_and_boost_cs2(timeout_secs: u64, affinity_mask: Option<usize>) -> bool {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if boost_cs2_process_native(affinity_mask) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(1500));
    }
    false
}

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub const BACKUP_FILE_NAME: &str = "optimizer_backup.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpInterfaceBackup {
    pub subkey_name: String,
    pub tcp_ack_frequency: Option<u32>,
    pub tcp_nodelay: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBackup {
    pub timestamp_unix: u64,
    pub power_plan_guid: Option<String>,
    pub tcp_interfaces: Vec<TcpInterfaceBackup>,
    pub system_responsiveness: Option<u32>,
    pub network_throttling_index: Option<u32>,
    pub win32_priority_separation: Option<u32>,
    pub disable_paging_executive: Option<u32>,
    pub app_capture_enabled: Option<u32>,
    pub game_dvr_enabled: Option<u32>,
    pub fullscreen_opt_entry: Option<String>,
    pub autoexec_backup_path: Option<String>,
}

impl SystemBackup {
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err("Backup file does not exist".to_string());
        }
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let backup: SystemBackup = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(backup)
    }

    pub fn capture_current_state(cs2_path: Option<&Path>) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let power_plan_guid = crate::system::get_active_power_plan_guid();

        let mut tcp_interfaces = Vec::new();
        #[cfg(not(windows))]
        {
            let _ = &mut tcp_interfaces;
        }
        #[cfg(windows)]
        {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let path = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces";
            if let Ok(root_key) = hklm.open_subkey_with_flags(path, KEY_READ) {
                for sub_name in root_key.enum_keys().flatten() {
                    let sub_path = format!(r"{}\{}", path, sub_name);
                    if let Ok(if_key) = hklm.open_subkey_with_flags(&sub_path, KEY_READ) {
                        let ack = if_key.get_value::<u32, _>("TcpAckFrequency").ok();
                        let nodelay = if_key.get_value::<u32, _>("TCPNoDelay").ok();
                        tcp_interfaces.push(TcpInterfaceBackup {
                            subkey_name: sub_name,
                            tcp_ack_frequency: ack,
                            tcp_nodelay: nodelay,
                        });
                    }
                }
            }
        }

        let (system_responsiveness, network_throttling_index) = crate::network::read_mmcss_state();
        let win32_priority_separation = crate::system::get_win32_priority_separation();
        let disable_paging_executive = crate::system::get_disable_paging_executive();
        let (app_capture_enabled, game_dvr_enabled) = crate::system::get_gamebar_state();

        let mut fullscreen_opt_entry = None;
        if let Some(cs2) = cs2_path {
            fullscreen_opt_entry = crate::system::get_fullscreen_optimizations_state(cs2);
        }

        SystemBackup {
            timestamp_unix: now,
            power_plan_guid,
            tcp_interfaces,
            system_responsiveness,
            network_throttling_index,
            win32_priority_separation,
            disable_paging_executive,
            app_capture_enabled,
            game_dvr_enabled,
            fullscreen_opt_entry,
            autoexec_backup_path: None,
        }
    }

    pub fn restore(&self, cs2_path: Option<&Path>) -> Vec<String> {
        let mut report = Vec::new();

        if let Some(ref guid) = self.power_plan_guid {
            if crate::system::set_power_plan_guid(guid) {
                report.push(format!("Power plan restored to: {}", guid));
            } else {
                report.push(format!("Failed to restore power plan: {}", guid));
            }
        }

        #[cfg(windows)]
        {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let base_path = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces";
            let mut restored_count = 0;
            for iface in &self.tcp_interfaces {
                let sub_path = format!(r"{}\{}", base_path, iface.subkey_name);
                if let Ok(key) = hklm.open_subkey_with_flags(&sub_path, KEY_SET_VALUE) {
                    match iface.tcp_ack_frequency {
                        Some(val) => { let _ = key.set_value("TcpAckFrequency", &val); }
                        None => { let _ = key.delete_value("TcpAckFrequency"); }
                    }
                    match iface.tcp_nodelay {
                        Some(val) => { let _ = key.set_value("TCPNoDelay", &val); }
                        None => { let _ = key.delete_value("TCPNoDelay"); }
                    }
                    restored_count += 1;
                }
            }
            report.push(format!("Restored TCP settings on {} interfaces", restored_count));
        }

        crate::network::restore_mmcss_state(self.system_responsiveness, self.network_throttling_index);
        report.push("Restored MMCSS SystemResponsiveness and NetworkThrottlingIndex".to_string());

        crate::system::restore_win32_priority_separation(self.win32_priority_separation);
        report.push(format!(
            "Restored Win32PrioritySeparation to {:?}",
            self.win32_priority_separation
        ));

        crate::system::restore_disable_paging_executive(self.disable_paging_executive);
        report.push(format!(
            "Restored DisablePagingExecutive to {:?}",
            self.disable_paging_executive
        ));

        crate::system::restore_gamebar_state(self.app_capture_enabled, self.game_dvr_enabled);
        report.push("Restored Xbox GameBar and GameDVR state".to_string());

        if let Some(cs2) = cs2_path {
            crate::system::restore_fullscreen_optimizations(cs2, self.fullscreen_opt_entry.as_deref());
            report.push("Restored Fullscreen Optimizations registry flags for cs2.exe".to_string());
        }

        if let Some(ref backup_str) = self.autoexec_backup_path {
            let backup_p = PathBuf::from(backup_str);
            if backup_p.exists() {
                if let Some(cs2) = cs2_path {
                    let target_cfg = cs2.join("game/csgo/cfg/autoexec.cfg");
                    if fs::copy(&backup_p, &target_cfg).is_ok() {
                        report.push(format!("Restored autoexec.cfg from {:?}", backup_p));
                    }
                }
            }
        }

        report
    }
}

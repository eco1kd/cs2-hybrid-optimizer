use std::fs;
use std::path::{Path, PathBuf};

pub fn get_folder_size_mb(path: &Path) -> f64 {
    if !path.exists() {
        return 0.0;
    }
    let mut total_bytes: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Ok(meta) = entry.metadata() {
                    total_bytes += meta.len();
                }
            } else if p.is_dir() {
                total_bytes += get_dir_bytes_recursive(&p);
            }
        }
    }
    (total_bytes as f64) / (1024.0 * 1024.0)
}

fn get_dir_bytes_recursive(dir: &Path) -> u64 {
    let mut bytes = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Ok(meta) = entry.metadata() {
                    bytes += meta.len();
                }
            } else if p.is_dir() {
                bytes += get_dir_bytes_recursive(&p);
            }
        }
    }
    bytes
}

pub fn clean_folder_contents(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let mut any_removed = false;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let item_path = entry.path();
            if item_path.is_file() || item_path.is_symlink() {
                if fs::remove_file(&item_path).is_ok() {
                    any_removed = true;
                }
            } else if item_path.is_dir() {
                if fs::remove_dir_all(&item_path).is_ok() {
                    any_removed = true;
                }
            }
        }
    }
    any_removed
}

pub struct CacheTarget {
    pub name: &'static str,
    pub path: PathBuf,
}

pub fn get_all_cache_targets(steam_path: Option<&Path>) -> Vec<CacheTarget> {
    let mut targets = Vec::new();

    let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let app_data = std::env::var_os("APPDATA").map(PathBuf::from);

    if let Some(local) = local_app_data {
        let subdirs = [
            ("D3DSCache", "D3DSCache"),
            ("NVIDIA DXCache", "NVIDIA/DXCache"),
            ("NVIDIA GLCache", "NVIDIA/GLCache"),
            ("AMD DxCache", "AMD/DxCache"),
            ("AMD DxcCache", "AMD/DxcCache"),
            ("AMD OglCache", "AMD/OglCache"),
            ("AMD VkCache", "AMD/VkCache"),
            ("Intel ShaderCache", "Intel/ShaderCache"),
        ];
        for (name, sub) in subdirs {
            targets.push(CacheTarget {
                name,
                path: local.join(sub),
            });
        }
    }

    if let Some(roaming) = app_data {
        targets.push(CacheTarget {
            name: "NVIDIA ComputeCache",
            path: roaming.join("NVIDIA/ComputeCache"),
        });
    }

    if let Some(steam) = steam_path {
        targets.push(CacheTarget {
            name: "Steam Shader 730",
            path: steam.join("steamapps/shadercache/730"),
        });
    }

    targets
}

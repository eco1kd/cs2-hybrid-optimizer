mod cache;
mod cs2;
mod memory;
mod network;
mod system;
mod timer;
mod types;
mod winapi_sys;

use colored::Colorize;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use types::{get_strings, Language};

const LOG_FILE_NAME: &str = "optimizer_log.txt";
const MAX_LOG_SIZE: u64 = 1024 * 1024;

fn get_exe_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn rotate_log(log_path: &Path) {
    if let Ok(meta) = fs::metadata(log_path) {
        if meta.len() > MAX_LOG_SIZE {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup = log_path.with_extension(format!("txt.{}.bak", timestamp));
            let _ = fs::rename(log_path, backup);
        }
    }
}

fn log(log_path: &Path, msg: &str) {
    let now = chrono_fmt_now();
    let line = format!("[{}] {}\n", now, msg);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = f.write_all(line.as_bytes());
    }
}

fn chrono_fmt_now() -> String {
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("UNIX:{}", dur)
}

fn elevate_uac(lang: Language) {
    #[cfg(windows)]
    unsafe {
        let exe = env::current_exe().unwrap_or_default();
        let exe_w: Vec<u16> = exe
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let op_w: Vec<u16> = "runas\0".encode_utf16().collect();
        let lang_arg = match lang {
            Language::En => "--lang=en\0",
            Language::Ru => "--lang=ru\0",
        };
        let param_w: Vec<u16> = lang_arg.encode_utf16().collect();
        let dir = get_exe_dir();
        let dir_w: Vec<u16> = dir
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        winapi_sys::ShellExecuteW(
            0,
            op_w.as_ptr(),
            exe_w.as_ptr(),
            param_w.as_ptr(),
            dir_w.as_ptr(),
            1,
        );
    }
    std::process::exit(0);
}

fn select_language() -> Language {
    for arg in env::args() {
        if arg == "--lang=en" {
            return Language::En;
        }
        if arg == "--lang=ru" {
            return Language::Ru;
        }
    }

    println!("============================================================");
    println!("  Select language / Выберите язык:");
    println!("============================================================");
    println!("  1 - English");
    println!("  2 - Русский");
    println!("============================================================");
    print!("👉 ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    if input.trim() == "1" {
        Language::En
    } else {
        Language::Ru
    }
}

fn main() {
    let exe_dir = get_exe_dir();
    let log_path = exe_dir.join(LOG_FILE_NAME);
    rotate_log(&log_path);

    let mut chosen_lang = None;
    for arg in env::args() {
        if arg == "--lang=en" {
            chosen_lang = Some(Language::En);
        } else if arg == "--lang=ru" {
            chosen_lang = Some(Language::Ru);
        }
    }

    let lang = chosen_lang.unwrap_or_else(|| {
        if !winapi_sys::is_admin() {
            select_language()
        } else {
            Language::Ru
        }
    });

    if !winapi_sys::is_admin() {
        elevate_uac(lang);
    }

    let txt = get_strings(lang);

    #[cfg(windows)]
    let _ = std::process::Command::new("cmd").args(["/C", "cls"]).status();

    log(&log_path, "==================================================");
    log(&log_path, &format!("Optimizer started (Rust) | lang={:?}", lang));

    println!("{}", "==============================================================".cyan().bold());
    println!("  {}", txt.title.cyan().bold());
    println!("{}", "==============================================================".cyan().bold());

    let (physical_cores, logical_threads) = cs2::detect_cpu_cores();
    let threads_for_cs2 = physical_cores;
    let launch_options = format!(
        "-high -nojoy -fullscreen -threads {} +fps_max 0 +exec autoexec",
        threads_for_cs2
    );

    println!(
        "\n  {}: {} Physical / {} Threads -> -threads {}",
        txt.cores_info.white(),
        physical_cores,
        logical_threads,
        threads_for_cs2
    );
    println!("  {}", txt.launch.yellow().bold());
    println!("  {}\n", launch_options.green().bold());
    println!("{}", "--------------------------------------------------------------".white());

    log(&log_path, &format!("CPU: {} physical / {} logical", physical_cores, logical_threads));
    log(&log_path, &format!("Launch options: {}", launch_options));

    let steam_path = cs2::get_steam_path();
    let cs2_path = cs2::get_cs2_path(steam_path.as_deref());

    if let Some(ref cs2) = cs2_path {
        log(&log_path, &format!("CS2 path detected: {:?}", cs2));
        println!("{}", txt.defender.cyan());
        if system::add_defender_exclusion(cs2) {
            println!("  {}: {:?}", txt.defender_ok.green(), cs2);
            log(&log_path, &format!("Defender exclusion added for {:?}", cs2));
        } else {
            println!("  {}", txt.defender_fail.yellow());
        }
    } else {
        log(&log_path, "CS2 path could not be detected in registry or default library");
    }

    println!("{}", txt.checking.cyan());
    let _ = std::process::Command::new("taskkill")
        .args(["/f", "/im", "cs2.exe"])
        .output();

    let cache_targets = cache::get_all_cache_targets(steam_path.as_deref());
    let mut total_before = 0.0;
    for target in &cache_targets {
        let size = cache::get_folder_size_mb(&target.path);
        total_before += size;
        if size > 0.0 {
            log(&log_path, &format!("Cache [{}] before: {:.2} MB", target.name, size));
        }
    }

    println!("{}", txt.call_gpu.cyan());
    for target in &cache_targets {
        if target.name.contains("GPU") || target.name.contains("Cache") {
            cache::clean_folder_contents(&target.path);
        }
    }

    println!("{}", txt.call_steam.cyan());
    if let Some(ref steam) = steam_path {
        let steam_shader = steam.join("steamapps/shadercache/730");
        cache::clean_folder_contents(&steam_shader);
    }

    println!("{}", txt.call_ram.cyan());
    let ram_ok = memory::purge_standby_list();
    log(&log_path, &format!("RAM standby list purge status: {}", ram_ok));

    println!("{}", txt.call_timer.cyan());
    let timer_res = timer::request_high_precision_timer();
    log(&log_path, &format!("Timer resolution requested: {:?}", timer_res));

    println!("\n{}", txt.success.green().bold());
    println!("{}", "--------------------------------------------------------------".white());

    let mut total_after = 0.0;
    for target in &cache_targets {
        total_after += cache::get_folder_size_mb(&target.path);
    }
    let actual_freed_mb = (total_before - total_after).max(0.0);

    println!("{}", txt.net_tune.cyan());
    match network::tune_tcp_latency() {
        Ok(count) => {
            println!("  {} ({} interfaces)", txt.net_tune_ok.green(), count);
            log(&log_path, &format!("TCP latency tuned on {} interfaces", count));
        }
        Err(e) => {
            println!("  {}: {}", txt.net_tune_fail.yellow(), e);
            log(&log_path, &format!("TCP tune error: {}", e));
        }
    }

    println!("{}", txt.sys_resp.cyan());
    if network::tune_mmcss().is_ok() {
        println!("  {}", txt.sys_resp_ok.green());
        log(&log_path, "MMCSS SystemResponsiveness set to 0, NetworkThrottling disabled");
    } else {
        println!("  {}", txt.sys_resp_fail.yellow());
    }

    println!("{}", txt.dns_flush.cyan());
    if system::flush_dns() {
        println!("  {}", txt.dns_ok.green());
        log(&log_path, "DNS cache flushed");
    } else {
        println!("  {}", txt.dns_fail.yellow());
    }

    println!("{}", txt.power_plan.cyan());
    if system::set_high_performance_power_plan() {
        println!("  {}", txt.power_ok.green());
        log(&log_path, "Power plan set to High Performance");
    } else {
        println!("  {}", txt.power_fail.yellow());
    }

    println!("{}", txt.gamebar.cyan());
    if system::disable_xbox_gamebar() {
        println!("  {}", txt.gamebar_ok.green());
        log(&log_path, "GameDVR / GameBar disabled");
    } else {
        println!("  {}", txt.gamebar_fail.yellow());
    }

    println!("{}", txt.priority_sep.cyan());
    if system::set_win32_priority_separation() {
        println!("  {}", txt.priority_sep_ok.green());
        log(&log_path, "Win32PrioritySeparation tuned (0x26)");
    } else {
        println!("  {}", txt.priority_sep_fail.yellow());
    }

    if let Some(ref cs2) = cs2_path {
        println!("{}", txt.autoexec.cyan());
        match cs2::generate_autoexec(cs2) {
            Ok(p) => {
                println!("  {}: {:?}", txt.autoexec_ok.green(), p);
                log(&log_path, &format!("autoexec.cfg created at {:?}", p));
            }
            Err(e) => {
                println!("  {}: {}", txt.autoexec_fail.yellow(), e);
            }
        }

        println!("{}", txt.fse.cyan());
        if system::disable_fullscreen_optimizations(cs2) {
            println!("  {}", txt.fse_ok.green());
            log(&log_path, "Fullscreen optimizations disabled for cs2.exe");
        } else {
            println!("  {}", txt.fse_fail.yellow());
        }
    }

    println!("{}", "--------------------------------------------------------------".white());

    println!("\n  ══ {} ══", txt.summary.magenta().bold());
    println!(
        "  {}: {:.1} MB (was: {:.1} MB)",
        txt.freed.green().bold(),
        actual_freed_mb,
        total_before
    );
    println!("  {}: {:?}", txt.log_saved.yellow(), log_path);
    println!("{}", "==============================================================".cyan().bold());

    log(&log_path, &format!("Cleanup summary: freed {:.1} MB", actual_freed_mb));
    log(&log_path, "==================================================");

    print!("\n  {}", txt.ask_launch);
    let _ = io::stdout().flush();
    let mut answer = String::new();
    let _ = io::stdin().read_line(&mut answer);
    let ans = answer.trim().to_lowercase();

    if ans.is_empty() || ans == "y" || ans == "yes" || ans == "д" || ans == "да" {
        println!("  {}", txt.launching.green().bold());
        let launched = cs2::launch_cs2(steam_path.as_deref(), &launch_options);
        log(&log_path, &format!("CS2 launched status: {}", launched));

        println!("  {}", txt.priority.cyan());
        let priority_ok_text = txt.priority_ok;
        let priority_fail_text = txt.priority_fail;
        std::thread::spawn(move || {
            if cs2::watch_and_boost_cs2(60) {
                println!("\n  {}", priority_ok_text.green().bold());
            } else {
                println!("\n  {}", priority_fail_text.yellow());
            }
        });
    }

    println!();
    println!("  {}", txt.exit.white());
    let mut exit_buf = String::new();
    let _ = io::stdin().read_line(&mut exit_buf);

    timer::restore_timer_resolution();
    log(&log_path, "Timer resolution restored. Exiting cleanly.");
}

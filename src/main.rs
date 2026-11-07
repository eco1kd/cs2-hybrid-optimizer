mod backup;
mod cache;
mod cs2;
mod memory;
mod network;
mod system;
mod timer;
mod types;
mod winapi_sys;

use backup::{SystemBackup, BACKUP_FILE_NAME};
use colored::Colorize;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use types::{get_strings, Language, RunMode};

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

fn elevate_uac(lang: Language, extra_args: &[String]) {
    #[cfg(not(windows))]
    {
        let _ = (lang, extra_args);
    }
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
            Language::En => "--lang=en",
            Language::Ru => "--lang=ru",
        };
        let mut full_params = lang_arg.to_string();
        for a in extra_args {
            full_params.push(' ');
            full_params.push_str(a);
        }
        let param_w: Vec<u16> = full_params
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
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

fn select_action_mode(txt: &types::Strings) -> RunMode {
    println!("\n{}", "============================================================".cyan());
    println!("  {}", txt.menu_title.cyan().bold());
    println!("{}", "============================================================".cyan());
    println!("  {}", txt.menu_opt1.green().bold());
    println!("  {}", txt.menu_opt2.yellow().bold());
    println!("  {}", txt.menu_opt3.blue().bold());
    println!("{}", "============================================================".cyan());
    print!("👉 {}", txt.menu_prompt);
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    match input.trim() {
        "2" => RunMode::RevertSettings,
        "3" => RunMode::CleanOnly,
        _ => RunMode::OptimizeAndLaunch,
    }
}

fn main() {
    let exe_dir = get_exe_dir();
    let log_path = exe_dir.join(LOG_FILE_NAME);
    rotate_log(&log_path);

    let mut chosen_lang = None;
    let mut chosen_mode = None;
    let mut silent = false;
    let mut extra_args = Vec::new();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--lang=en" => chosen_lang = Some(Language::En),
            "--lang=ru" => chosen_lang = Some(Language::Ru),
            "--revert" | "-r" => {
                chosen_mode = Some(RunMode::RevertSettings);
                extra_args.push(arg);
            }
            "--clean-only" => {
                chosen_mode = Some(RunMode::CleanOnly);
                extra_args.push(arg);
            }
            "--silent" | "--auto" => {
                silent = true;
                extra_args.push(arg);
            }
            _ => {}
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
        elevate_uac(lang, &extra_args);
    }

    let txt = get_strings(lang);

    #[cfg(windows)]
    let _ = std::process::Command::new("cmd").args(["/C", "cls"]).status();

    log(&log_path, "==================================================");
    log(&log_path, &format!("Optimizer started (Rust) | lang={:?}", lang));

    println!("{}", "==============================================================".cyan().bold());
    println!("  {}", txt.title.cyan().bold());
    println!("{}", "==============================================================".cyan().bold());

    let mode = chosen_mode.unwrap_or_else(|| {
        if silent {
            RunMode::OptimizeAndLaunch
        } else {
            select_action_mode(&txt)
        }
    });

    let steam_path = cs2::get_steam_path();
    let cs2_path = cs2::get_cs2_path(steam_path.as_deref());

    match mode {
        RunMode::RevertSettings => {
            println!("\n  ══ {} ══\n", txt.revert_title.yellow().bold());
            println!("{}", txt.revert_start.cyan());
            log(&log_path, "Initiating system settings restoration (revert mode)");

            let backup_path = exe_dir.join(BACKUP_FILE_NAME);
            match SystemBackup::load_from_file(&backup_path) {
                Ok(backup) => {
                    let reports = backup.restore(cs2_path.as_deref());
                    for rep in reports {
                        println!("  {} {}", "[ OK ]".green(), rep);
                        log(&log_path, &format!("Revert: {}", rep));
                    }
                    println!("\n  {}\n", txt.revert_done.green().bold());
                    log(&log_path, "Revert completed successfully.");
                }
                Err(e) => {
                    println!("  {}: {}", txt.revert_not_found.yellow(), e);
                    log(&log_path, &format!("Revert failed: {}", e));
                }
            }

            println!("  {}", txt.exit.white());
            let mut exit_buf = String::new();
            let _ = io::stdin().read_line(&mut exit_buf);
            return;
        }

        RunMode::CleanOnly => {
            println!("\n[ .. ] Cleaning GPU/Steam caches and RAM Standby List only...");
            let cache_targets = cache::get_all_cache_targets(steam_path.as_deref());
            let mut total_before = 0.0;
            for target in &cache_targets {
                total_before += cache::get_folder_size_mb(&target.path);
            }

            for target in &cache_targets {
                cache::clean_folder_contents(&target.path);
            }
            if let Some(ref steam) = steam_path {
                let steam_shader = steam.join("steamapps/shadercache/730");
                cache::clean_folder_contents(&steam_shader);
            }
            let _ = memory::purge_standby_list();

            let mut total_after = 0.0;
            for target in &cache_targets {
                total_after += cache::get_folder_size_mb(&target.path);
            }
            let freed = (total_before - total_after).max(0.0);

            println!("  [ OK ] Cache cleaned: {:.1} MB", freed);
            log(&log_path, &format!("CleanOnly completed: freed {:.1} MB", freed));

            println!("\n  {}", txt.exit.white());
            let mut exit_buf = String::new();
            let _ = io::stdin().read_line(&mut exit_buf);
            return;
        }

        RunMode::OptimizeAndLaunch => {
        }
    }

    timer::register_console_ctrl_handler();

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

    let backup_path = exe_dir.join(BACKUP_FILE_NAME);
    let mut system_snapshot = if !backup_path.exists() {
        let snapshot = SystemBackup::capture_current_state(cs2_path.as_deref());
        log(&log_path, "Captured pre-optimizer system state snapshot");
        Some(snapshot)
    } else {
        None
    };

    println!("{}", txt.checking.cyan());
    let killed = cs2::kill_lingering_cs2_native();
    if killed > 0 {
        log(&log_path, &format!("Killed {} lingering cs2.exe instances natively", killed));
    }

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

    println!("{}", txt.paging_exec.cyan());
    match system::enable_disable_paging_executive() {
        system::PagingExecResult::Applied => {
            println!("  {}", txt.paging_exec_ok.green());
            log(&log_path, "DisablePagingExecutive enabled (RAM >= 16GB, kernel locked in RAM)");
        }
        system::PagingExecResult::SkippedLowRam(ram) => {
            println!("  {} ({:.1} GB RAM)", txt.paging_exec_skip_ram.yellow(), ram);
            log(&log_path, &format!("DisablePagingExecutive skipped: detected {:.1} GB RAM (< 16GB threshold)", ram));
        }
        system::PagingExecResult::Failed => {
            println!("  {}", txt.paging_exec_fail.yellow());
        }
    }

    if let Some(ref cs2) = cs2_path {
        println!("{}", txt.autoexec.cyan());
        match cs2::generate_autoexec(cs2) {
            Ok((cfg_p, backup_p)) => {
                println!("  {}: {:?}", txt.autoexec_ok.green(), cfg_p);
                log(&log_path, &format!("autoexec.cfg updated at {:?}", cfg_p));
                if let Some(ref b) = backup_p {
                    if let Some(ref mut snap) = system_snapshot {
                        snap.autoexec_backup_path = Some(b.to_string_lossy().to_string());
                    }
                }
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

    if let Some(ref snap) = system_snapshot {
        if snap.save_to_file(&backup_path).is_ok() {
            println!("  {}", txt.backup_created.green());
            log(&log_path, &format!("Pre-optimizer snapshot saved to {:?}", backup_path));
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

    let affinity_mask = cs2::calculate_pcore_affinity_mask(physical_cores, logical_threads);

    let mut should_launch = silent;
    if !silent {
        print!("\n  {}", txt.ask_launch);
        let _ = io::stdout().flush();
        let mut answer = String::new();
        let _ = io::stdin().read_line(&mut answer);
        let ans = answer.trim().to_lowercase();
        should_launch = ans.is_empty() || ans == "y" || ans == "yes" || ans == "д" || ans == "да";
    }

    if should_launch {
        println!("  {}", txt.launching.green().bold());
        let launched = cs2::launch_cs2(steam_path.as_deref(), &launch_options);
        log(&log_path, &format!("CS2 launched status: {}", launched));

        println!("  {}", txt.priority.cyan());
        let priority_ok_text = txt.priority_ok;
        let priority_fail_text = txt.priority_fail;
        std::thread::spawn(move || {
            if cs2::watch_and_boost_cs2(90, affinity_mask) {
                println!("\n  {}", priority_ok_text.green().bold());
            } else {
                println!("\n  {}", priority_fail_text.yellow());
            }
        });
    }

    if !silent {
        println!();
        println!("  {}", txt.exit.white());
        let mut exit_buf = String::new();
        let _ = io::stdin().read_line(&mut exit_buf);
    }

    timer::restore_timer_resolution();
    log(&log_path, "Timer resolution restored. Exiting cleanly.");
}

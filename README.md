<div align="center">

# CS2 Optimizer (Rust Edition)

### High-Performance Native Counter-Strike 2 Optimizer

![GitHub release](https://img.shields.io/github/v/release/eco1kd/cs2-hybrid-optimizer?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-blue?style=for-the-badge)
![Language](https://img.shields.io/badge/Language-Rust%20🦀-orange?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge)
![CS2](https://img.shields.io/badge/Counter--Strike%202-Optimized-orange?style=for-the-badge)

### Eliminate Stutters - Stabilize FPS - Crisp Sub-Tick Hitreg - Standalone ~1 MB Binary

| Select Language |
|:---:|
| **English** (current) &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; [**Russian**](./README_RU.md) |

</div>

---

## Overview

**CS2 Optimizer** is a native, lightweight, standalone Windows utility written in **Rust** designed to eliminate frame-time spikes, optimize system latency, and prepare Windows for **Counter-Strike 2**.

Rewritten from the ground up in Rust, it delivers maximum performance without requiring Python runtimes, external DLLs, or slow unpackers:
- **Single ~1 MB executable**
- **Instant startup (~5ms)**
- **Zero antivirus false-positives**
- **Direct Windows NT kernel syscalls**

The primary goal is to improve:
- 0.1% and 1% Low FPS consistency
- Input and network latency (Sub-Tick hit registration)
- Shader compilation stutter prevention
- RAM Standby List freeing without paging to disk

---

## Architecture

| Component | Role | Technologies |
| --- | --- | --- |
| Core Engine | Memory-safe, zero-allocation native Windows execution | `Rust 2021`, `windows-sys`, `winreg`, `ntdll`, `kernel32` |
| NT Kernel Layer | Standby List purge, High-Precision ~0.5ms timer lock | Native NT API (`NtSetSystemInformation`, `NtSetTimerResolution`) |
| Network Layer | Nagle's algorithm disable (`TcpAckFrequency = 1`, `TCPNoDelay = 1`) & MMCSS profile uncapping | Windows Registry (`Tcpip\Parameters\Interfaces`, `SystemProfile`) |
| Game Integration | Steam library discovery, Sub-Tick autoexec, Defender exclusions | Steam VDF parser, `steam.exe -applaunch 730`, Win32 Toolhelp32 |

---

## Features

### Cache & Memory
- **GPU Cache Reset**: Purges D3DSCache, NVIDIA DXCache/GLCache/ComputeCache, AMD DxCache/DxcCache/OglCache/VkCache, and Intel ShaderCache.
- **Steam Shader Cache Cleanup**: Cleans AppID 730 shader data in `steamapps/shadercache/730`.
- **RAM Standby List Purge**: Calls NT API with `SeProfileSingleProcessPrivilege` to flush standby memory pages.
- **High-Precision System Timer**: Requests and locks timer resolution to ~0.5ms (instead of default 15.6ms) for the duration of the game session.

### System & Network
- **TCP Latency Tuning**: Disables packet buffering (Nagle's algorithm) on all network adapters for instantaneous Sub-Tick hit registration.
- **MMCSS System Profile**: Sets `SystemResponsiveness = 0` and `NetworkThrottlingIndex = 0xffffffff`.
- **Win32PrioritySeparation**: Sets `0x26` for maximum foreground process responsiveness.
- **Power Plan**: Locks Windows into the High Performance power scheme.
- **Fullscreen Optimizations**: Disables hybrid fullscreen DWM optimizations for `cs2.exe`.
- **GameDVR**: Disables Xbox GameBar background recording hooks.
- **DNS Flush**: Clears DNS cache before connecting to matchmaking.

### CS2 Launch & Background Watcher
- **Multi-Drive Library Support**: Automatically detects CS2 across any SSD drive or library folder.
- **Sub-Tick `autoexec.cfg`**: Writes modern Sub-Tick commands (`engine_low_latency_sleep_after_client_tick true`, `cl_net_buffer_ticks 0`, rates) while **preserving backups** of existing configs.
- **Native Steam AppLaunch**: Starts CS2 via `steam.exe -applaunch 730` ensuring all parameters are passed.
- **Native Background Booster**: Thread polls for `cs2.exe` and elevates its process priority to `HIGH_PRIORITY_CLASS`.

---

## How To Build

### Requirements
* [Rust](https://rustup.rs/) (cargo & rustc)

### One-Click Build (Windows)
Double-click `build.bat` or run:
```cmd
cargo build --release
```
The resulting binary will be located at:
```text
target/release/cs2-optimizer.exe
```

---

## How To Use

1. Download `cs2-optimizer.exe` from [Releases](../../releases).
2. Right-click and choose **Run as Administrator**.
3. Choose your language (English / Russian).
4. Review the optimization summary and press `Y` or `Enter` to launch Counter-Strike 2.
5. Keep the console window open during your match to maintain the ~0.5ms high-precision timer.

---

## Safety

This project:
- Does NOT inject code into the CS2 game process
- Does NOT hook VAC modules or modify game memory
- Does NOT modify game binaries
- 100% compliant with Valve Anti-Cheat (VAC)

---

## License

MIT License

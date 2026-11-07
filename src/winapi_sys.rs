#[cfg(windows)]
use std::ffi::c_void;

#[cfg(windows)]
pub use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, BOOL, HANDLE, INVALID_HANDLE_VALUE, LUID,
    },
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueA, TOKEN_ADJUST_PRIVILEGES,
        TOKEN_PRIVILEGES, TOKEN_QUERY,
    },
    System::{
        Console::SetConsoleCtrlHandler,
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
            TH32CS_SNAPPROCESS,
        },
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        Threading::{
            GetCurrentProcess, OpenProcess, OpenProcessToken, SetPriorityClass,
            SetProcessAffinityMask, TerminateProcess, HIGH_PRIORITY_CLASS,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION, PROCESS_TERMINATE,
        },
    },
};

#[cfg(windows)]
#[link(name = "shell32")]
extern "system" {
    pub fn IsUserAnAdmin() -> i32;

    pub fn ShellExecuteW(
        hwnd: isize,
        lpOperation: *const u16,
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShowCmd: i32,
    ) -> isize;
}

#[cfg(windows)]
#[link(name = "ntdll")]
extern "system" {
    pub fn NtSetSystemInformation(
        SystemInformationClass: u32,
        SystemInformation: *mut c_void,
        SystemInformationLength: u32,
    ) -> i32;

    pub fn NtQueryTimerResolution(
        MaximumResolution: *mut u32,
        MinimumResolution: *mut u32,
        CurrentResolution: *mut u32,
    ) -> i32;

    pub fn NtSetTimerResolution(
        DesiredResolution: u32,
        SetResolution: u8,
        CurrentResolution: *mut u32,
    ) -> i32;
}

pub fn is_admin() -> bool {
    #[cfg(windows)]
    unsafe {
        IsUserAnAdmin() != 0
    }
    #[cfg(not(windows))]
    {
        true
    }
}

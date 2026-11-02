#[cfg(windows)]
use crate::winapi_sys::*;
#[cfg(windows)]
use std::ffi::CString;

#[cfg(windows)]
unsafe fn enable_privilege(token: HANDLE, priv_name: &str) -> bool {
    let Ok(c_name) = CString::new(priv_name) else {
        return false;
    };
    let mut luid = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    if LookupPrivilegeValueA(std::ptr::null(), c_name.as_ptr() as *const u8, &mut luid) == 0 {
        return false;
    }

    let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();
    tp.PrivilegeCount = 1;
    tp.Privileges[0].Luid = luid;
    tp.Privileges[0].Attributes = 0x00000002;

    AdjustTokenPrivileges(
        token,
        0,
        &mut tp,
        std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    GetLastError() == 0
}

pub fn purge_standby_list() -> bool {
    #[cfg(windows)]
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return false;
        }

        enable_privilege(token, "SeProfileSingleProcessPrivilege");
        enable_privilege(token, "SeIncreaseQuotaPrivilege");
        CloseHandle(token);

        let sys_info_class = 80u32;
        let mut cmd = 4u32;
        let status = NtSetSystemInformation(
            sys_info_class,
            &mut cmd as *mut u32 as *mut std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );

        status == 0
    }
    #[cfg(not(windows))]
    {
        true
    }
}

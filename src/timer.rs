#[cfg(windows)]
use crate::winapi_sys::*;

pub fn request_high_precision_timer() -> Option<f64> {
    #[cfg(windows)]
    unsafe {
        let mut max_res = 0u32;
        let mut min_res = 0u32;
        let mut cur_res = 0u32;

        if NtQueryTimerResolution(&mut max_res, &mut min_res, &mut cur_res) != 0 {
            return None;
        }

        let mut actual = 0u32;
        let status = NtSetTimerResolution(min_res, 1, &mut actual);
        if status == 0 {
            Some(min_res as f64 / 10000.0)
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        Some(0.5)
    }
}

pub fn restore_timer_resolution() {
    #[cfg(windows)]
    unsafe {
        let mut actual = 0u32;
        NtSetTimerResolution(0, 0, &mut actual);
    }
}

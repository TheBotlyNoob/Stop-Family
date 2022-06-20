use std::{env, ffi::CString, ptr};
use windows::{
    core::PSTR,
    Win32::{
        Foundation::BOOL,
        System::Threading::{CreateProcessAsUserA, STARTUPINFOA},
        UI::Shell::IsUserAnAdmin,
    },
};

/// Returns true if the current process has admin rights, otherwise false.
pub fn is_elevated() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}

pub fn elevate() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = CString::new(env::current_exe()?.to_str().ok_or(crate::NoneErr)?)?.as_ptr() as _;
    if unsafe {
        !CreateProcessAsUserA(
            None,
            None,
            PSTR(cmd),
            ptr::null(),
            ptr::null(),
            BOOL(1),
            0,
            ptr::null(),
            None,
            &STARTUPINFOA {
                lpTitle: PSTR("dasd"),
                cb: std::mem::size_of::<STARTUPINFOA>() as _,
                lpReserved: PSTR(ptr::null_mut()),
                lpDesktop: PSTR(ptr::null_mut()),
            },
            ptr::null_mut(),
        )
        .as_bool()
    } {
        Err(Box::new(std::io::Error::last_os_error()))
    } else {
        Ok(())
    }
}

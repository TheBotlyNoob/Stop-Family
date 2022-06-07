use std::ffi::CString;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HANDLE,
        System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE},
        UI::{
            Shell::{
                IsUserAnAdmin, ShellExecuteExA, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS,
                SEE_MASK_NO_CONSOLE, SHELLEXECUTEINFOA,
            },
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    },
};

/// Returns true if the current process has admin rights, otherwise false.
pub fn is_elevated() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}

pub fn elevate() -> Result<(), Box<dyn std::error::Error>> {
    let lp_file = CString::new(std::env::current_exe()?.to_str().unwrap())?;
    let lp_verb = CString::new("runas")?;
    let lp_parameters = CString::new(std::env::args().skip(1).collect::<Vec<_>>().join(" "))?;
    let lp_directory = CString::new(std::env::current_dir()?.to_str().unwrap())?;
    let mut opts = SHELLEXECUTEINFOA {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOA>() as u32,
        lpVerb: PCSTR(lp_verb.as_ptr() as *const u8),
        lpFile: PCSTR(lp_file.as_ptr() as *const u8),
        lpParameters: PCSTR(lp_parameters.as_ptr() as *const u8),
        lpDirectory: PCSTR(lp_directory.as_ptr() as *const u8),
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC | SEE_MASK_NO_CONSOLE,
        nShow: SW_SHOWNORMAL.0 as i32,
        ..Default::default()
    };

    unsafe {
        CoInitializeEx(
            std::ptr::null(),
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        )?
    };

    if unsafe { !ShellExecuteExA(&mut opts).as_bool() } || opts.hProcess == HANDLE(0) {
        return Err(Box::new(std::io::Error::last_os_error()));
    };

    Ok(())
}

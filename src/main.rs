use std::{ffi::OsString, os::windows::prelude::OsStringExt, path::PathBuf, ptr};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HANDLE, LUID},
        Globalization::lstrlenW,
        Security::{
            AdjustTokenPrivileges, LookupPrivilegeValueA, LUID_AND_ATTRIBUTES,
            SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
        UI::Shell::{FOLDERID_RoamingAppData, IsUserAnAdmin, SHGetKnownFolderPath, KF_FLAG_CREATE},
    },
};

mod scheduled_tasks;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static BIN: &[u8] = include_bytes!(env!("CARGO_BIN_FILE_BIN"));

#[cfg(not(target_os = "windows"))]
compile_error!("This program is only intended to be run on Windows.");

fn main() -> Result<()> {
    if unsafe { !IsUserAnAdmin().as_bool() } {
        println!("[!] This program must be run as an administrator.");
        std::process::exit(1);
    } else {
        println!("[+] Elevated to administrator privileges.");

        // this allows us to create an administrator scheduled task
        {
            let mut process_token = HANDLE::default();

            unsafe {
                OpenProcessToken(
                    GetCurrentProcess(),
                    TOKEN_ADJUST_PRIVILEGES,
                    &mut process_token,
                );
            }

            let mut luid = LUID::default();

            unsafe {
                LookupPrivilegeValueA(None, "SeRestorePrivilege", &mut luid);
            }

            let mut new_state = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }; 1],
            };

            unsafe {
                AdjustTokenPrivileges(
                    process_token,
                    false,
                    &mut new_state as *mut _ as *mut _,
                    0,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
            }
        }

        let appdata = unsafe {
            let appdata =
                SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_CREATE.0 as _, None)?;
            let len = lstrlenW(PCWSTR(appdata.0)) as usize;
            let appdata = std::slice::from_raw_parts(appdata.0, len);
            PathBuf::from(OsString::from_wide(appdata))
        };
        let bin_path = &appdata.join("stop-family.exe");

        // make sure the folder exists
        std::fs::create_dir_all(&appdata)?;

        // copy the binary to the appdata folder
        // the result is ignored because it's not important if the file already exists
        let _ = std::fs::write(bin_path, BIN);

        // we do this so that if there's a Windows update, or the file gets restored,
        // we still make sure it's gone.
        scheduled_tasks::create_task(r"\Stop-Family", bin_path.to_str().unwrap(), true)?;

        scheduled_tasks::run_task(r"\Stop-Family")?;

        println!("[+] Finished. Closing in 5 seconds.");
        std::thread::sleep(std::time::Duration::from_secs(5));

        Ok(())
    }
}

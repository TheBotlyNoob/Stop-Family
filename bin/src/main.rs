#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ptr;
use windows::Win32::{
    Foundation::{HANDLE, LUID},
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueA, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
    UI::Shell::IsUserAnAdmin,
};

mod kill;

const WPCMON: &str = "WPCMon.exe";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(not(target_os = "windows"))]
compile_error!("This program is only intended to be run on Windows.");

fn main() -> Result<()> {
    if unsafe { !IsUserAnAdmin().as_bool() } {
        println!("[!] This program must be run as an administrator.");
        std::process::exit(1);
    } else {
        println!("[+] Elevated to administrator privileges.");

        // this allows us to write to the System32 folder
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

        println!("[+] Killing WPCMon...");
        kill::by_name(WPCMON)?;

        let mut exit_code = 0;

        let wpcmon_path = format!(r"C:\Windows\System32\{}", WPCMON);
        if let Err(e) = std::fs::rename(&wpcmon_path, format!("{}.bak", wpcmon_path)) {
            if e.kind() != std::io::ErrorKind::NotFound {
                println!("[!] Failed to delete {wpcmon_path}: {e:#?}.");
                exit_code = e.raw_os_error().unwrap_or(1);
            }
        } else {
            println!("[+] Deleted {wpcmon_path}.");
        }

        println!("[+] Finished. Closing in 5 seconds.");
        std::thread::sleep(std::time::Duration::from_secs(5));

        std::process::exit(exit_code);
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ptr;
use walkdir::WalkDir;
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

        for wpc in WalkDir::new(r"C:\Windows\System32")
            .into_iter()
            .filter_map(|r| r.ok())
            .filter(|r| {
                r.file_name().to_str().map(|s| {
                    let s = s.to_lowercase();
                    s.starts_with("wpc") && !s.ends_with("bak")
                }) == Some(true)
            })
        {
            kill::by_name(wpc.file_name().to_str().unwrap())?;

            if let Err(e) = std::fs::rename(&wpc.path(), wpc.path().with_extension("bak")) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    println!(
                        "[!] Failed to rename {}: {e}.",
                        wpc.file_name().to_string_lossy()
                    );
                }
            } else {
                println!("[+] Renamed {}.", wpc.file_name().to_string_lossy());
            }
        }

        println!("[+] Finished. Closing in 5 seconds.");
        std::thread::sleep(std::time::Duration::from_secs(5));

        Ok(())
    }
}

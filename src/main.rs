use std::path::Path;
use windows::{
    core::BSTR,
    s,
    Win32::{
        Foundation::{HANDLE, LUID},
        Security::{
            AdjustTokenPrivileges, LookupPrivilegeValueA, LUID_AND_ATTRIBUTES,
            SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
        },
        System::{
            Com::{CoCreateInstance, CoInitialize, CLSCTX_INPROC_SERVER, VARIANT},
            TaskScheduler::{ITaskService, TaskScheduler},
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
        UI::Shell::IsUserAnAdmin,
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The configuration directory for the Family Safety applications.
static CONFIG_DIR: &str = r"C:\ProgramData\Microsoft\Windows\Parental Controls\settings";
/// The path to the `FamilySafetyRefreshTask` task.
static REFRESH_TASK: &str = r"\Microsoft\Windows\Shell\FamilySafetyRefreshTask";
/// The path to the `FamilySafetyMonitor` task.
static MONITOR_TASK: &str = r"\Microsoft\Windows\Shell\FamilySafetyMonitor";

#[cfg(not(target_os = "windows"))]
compile_error!("This program is only intended to be run on Windows.");

fn main() -> Result<()> {
    if unsafe { !IsUserAnAdmin().as_bool() } {
        // TODO: non-elevated mode
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
                LookupPrivilegeValueA(None, s!("SeRestorePrivilege"), &mut luid);
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
                    Some(&mut new_state as *mut _ as *mut _),
                    0,
                    None,
                    None,
                );
            }
        }

        disable_task(REFRESH_TASK)?;
        disable_task(MONITOR_TASK)?;

        if std::fs::remove_dir_all(CONFIG_DIR).is_err() {
            println!("[!] Failed to remove configuration directory.");
        }

        println!("[+] Finished. Closing in 5 seconds.");
        std::thread::sleep(std::time::Duration::from_secs(5));

        Ok(())
    }
}

/// Disables a scheduled task.
pub fn disable_task(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    let task_folder = path.parent().unwrap().to_str().unwrap();
    let task_name = path.file_name().unwrap().to_str().unwrap();

    println!("[+] Disabling the {} task...", path.display());

    unsafe { CoInitialize(None)? };

    let task_service =
        unsafe { CoCreateInstance::<_, ITaskService>(&TaskScheduler, None, CLSCTX_INPROC_SERVER)? };

    unsafe {
        task_service.Connect(
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
        )?;
    }

    let task_folder = unsafe { task_service.GetFolder(&BSTR::from(task_folder))? };
    let task = unsafe { task_folder.GetTask(&BSTR::from(task_name))? };

    unsafe {
        task.Stop(0)?;
    }

    // remove the task from the task scheduler
    unsafe {
        task_folder.DeleteTask(&BSTR::from(task_name), 0)?;
    }

    Ok(())
}

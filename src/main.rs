use std::fs;

mod admin;
mod filesystem;
mod kill;

const WPCMON: &str = "wpcmon.exe";

const WPCMON_PATH: &str = "C:/Windows/System32/wpcmon.exe";

#[cfg(not(target_os = "windows"))]
compile_error!("This program is only intended to be run on Windows.");

#[cfg(target_os = "windows")]
fn main() {
    if !admin::is_elevated() {
        println!("[!] Elevating to administrator privileges. Please accept the UAC prompt.");
        if let Err(e) = admin::elevate() {
            println!("[!] Failed to elevate to administrator privileges: {e:#?}.");
            std::thread::sleep(std::time::Duration::from_secs(5));
            std::process::exit(1);
        }
    } else {
        println!("[+] Elevated to administrator privileges.");

        // kill wpcmon before we delete it
        kill::by_name(WPCMON);

        if let Err(e) = fs::remove_file(WPCMON_PATH) {
            println!("[!] Failed to delete {WPCMON_PATH}: {e:#?}.");
        } else {
            println!("[+] Deleted {WPCMON_PATH}.");
        }
    }
}

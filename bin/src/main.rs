#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notify::Watcher;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(not(target_os = "windows"))]
compile_error!("This program is only intended to be run on Windows.");

/// The configuration directory for the Family Safety applications.
static CONFIG_DIR: &str = r"C:\ProgramData\Microsoft\Windows\Parental Controls\settings";

fn main() -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify::watcher(tx, std::time::Duration::from_secs(10))?;

    let _ = delete_dir_files(CONFIG_DIR);

    println!("[+] Watching {} for changes...", CONFIG_DIR);
    watcher.watch(CONFIG_DIR, notify::RecursiveMode::NonRecursive)?;

    loop {
        if rx.recv().is_ok() {
            println!("[+] Configuration directory changed.");
            let _ = delete_dir_files(CONFIG_DIR);
        }
    }
}

fn delete_dir_files(dir: impl AsRef<std::path::Path>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        std::fs::remove_file(&entry?.path())?;
    }
    Ok(())
}

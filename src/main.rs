mod admin;
use std::{
  env, error::Error, fs::copy, path::Path, process::Command, result::Result as _Result, thread,
  time::Duration,
};
use sysinfo::{ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};

/// The Family Safety Moniter: https://strontic.github.io/xcyclopedia/library/WpcMon.exe-3BF899D6B668CAAD061E7D7FBE56A5A1.html
const WPCMON: &str = "WpcMon.exe";
/// The Family Safety Token Authenticatior: https://strontic.github.io/xcyclopedia/library/WpcTok.exe-09013BC3AA26A23515971B793AF73CF2.html
const WPCTOK: &str = "WpcTok.exe";

pub type Result<T> = _Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result<()> {
  admin::elevate()?;

  _main()
}

fn _main() -> Result<()> {
  loop {
    for process in System::new_with_specifics(
      RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    )
    .process_by_name(WPCMON)
    {
      process.kill();
      println!("Killed {}", WPCMON);
    }

    for _ in System::new_with_specifics(
      RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    )
    .process_by_name(WPCTOK)
    {
      thread::spawn(|| {
        let prev_system = System::new_with_specifics(
          RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
        );
        let prev_processes = prev_system.processes();
        thread::sleep(Duration::from_millis(55));

        let current_system = System::new_with_specifics(
          RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
        );
        let current_processes = current_system.processes();
        if prev_processes.len() != current_processes.len() {
          for (pid, process) in prev_processes {
            if !current_processes.contains_key(pid) && {
              let exe = process.exe().to_string_lossy();
              exe.contains(r":\")
                && !exe
                  .to_uppercase()
                  .starts_with(&env::var("SystemRoot").unwrap())
                && !process
                  .exe()
                  .file_name()
                  .unwrap()
                  .to_string_lossy()
                  .starts_with("__COPY__.")
            } {
              println!("Restarting any killed apps...");

              let new_file_path = format!(
                r"{}\__COPY__.{}",
                process
                  .exe()
                  .parent()
                  .unwrap_or(Path::new(&env::var("Temp").unwrap()))
                  .display(),
                process.exe().file_name().unwrap().to_string_lossy()
              );
              println!("{}", new_file_path);
              copy(process.exe(), &new_file_path).ok();
              Command::new(new_file_path)
                .args(process.cmd().iter().skip(1))
                .current_dir(process.cwd())
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
            }
          }
        }
      });
    }
  }
}

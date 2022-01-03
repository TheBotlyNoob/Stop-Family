mod admin;
mod perms;

use glob::{glob_with, MatchOptions};
use std::{env, fs::rename};

const GLOB_OPTIONS: MatchOptions = MatchOptions {
  case_sensitive: false,
  require_literal_separator: false,
  require_literal_leading_dot: true
};

fn main() {
  admin::elevate();

  _main();
}

fn _main() {
  for file in glob_with(
    &format!(r"{}\**\Wpc[MT]o[nk].exe", env::var("SystemRoot").unwrap()),
    GLOB_OPTIONS
  )
  .unwrap()
  .filter_map(Result::ok)
  {
    let original_owner = perms::become_owner(&file);
    let original_owner = original_owner.owner().unwrap();

    rename(
      &file,
      format!(
        r"{}\__{}",
        file.parent().unwrap().display(),
        file.file_name().unwrap().to_string_lossy()
      )
    )
    .unwrap();

    perms::set_owner(&file, original_owner);
  }

  loop {}
}

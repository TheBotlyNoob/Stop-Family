mod admin;
mod perms;
use glob::{glob_with, MatchOptions};
use std::{
  env,
  fs::{rename, File},
};

const GLOB_OPTIONS: MatchOptions = MatchOptions {
  case_sensitive: false,
  require_literal_separator: false,
  require_literal_leading_dot: true,
};

fn main() {
  admin::elevate();

  _main();
}

fn _main() {
  for file in glob_with(
    &format!(r"{}\**\Wpc[MT]o[nk].exe", env::var("SystemRoot").unwrap()),
    GLOB_OPTIONS,
  )
  .unwrap()
  .filter_map(Result::ok)
  {
    perms::become_owner(File::open(&file).unwrap());

    rename(
      &file,
      format!(
        r"{}\__{}",
        file.parent().unwrap().display(),
        file.file_name().unwrap().to_string_lossy()
      ),
    )
    .unwrap();
  }
}

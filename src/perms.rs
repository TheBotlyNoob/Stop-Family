use std::{ffi::OsStr, fs::File, mem, os::windows::io::IntoRawHandle, ptr};
use winapi::{shared::lmcons::UNLEN, um::winbase::GetUserNameA};
use windows_permissions::wrappers::LookupAccountName;

pub fn become_owner(file: File) {
  let mut current_user = (0..=(UNLEN + 1)).map(|_| 0).collect::<Vec<u8>>();

  unsafe {
    GetUserNameA(
      current_user.as_mut_ptr() as *mut i8,
      current_user.len() as *mut u32,
    );
  }

  println!("wdd");

  let current_user = String::from_utf8_lossy(&current_user).to_string();

  println!("{}", current_user);

  let file_handle = file.into_raw_handle();

  let (sid, _, _) = LookupAccountName(Option::<&OsStr>::None, current_user).unwrap();
}

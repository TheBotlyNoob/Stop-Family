use std::{env, ffi::CString, io::Error, mem, process::exit, ptr};

use winapi::um::{
  handleapi::CloseHandle,
  processthreadsapi::{GetCurrentProcess, OpenProcessToken},
  securitybaseapi::GetTokenInformation,
  shellapi::ShellExecuteA,
  winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY}
};

fn is_elevated() -> bool {
  unsafe {
    let mut handle = ptr::null_mut();
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) != 0 {
      let mut elevation = TOKEN_ELEVATION::default();
      let mut ret_size = mem::size_of::<TOKEN_ELEVATION>() as u32;
      // The weird looking repetition of `as *mut _` is casting the reference to a c_void pointer.
      let elevated = if GetTokenInformation(
        handle,
        TokenElevation,
        &mut elevation as *mut _ as *mut _,
        ret_size,
        &mut ret_size
      ) != 0
      {
        elevation.TokenIsElevated != 0
      } else {
        false
      };

      CloseHandle(handle);

      elevated
    } else {
      false
    }
  }
}

pub fn elevate() {
  if !is_elevated() {
    unsafe {
      ShellExecuteA(
        ptr::null_mut(),
        CString::new("RunAs").as_ref().unwrap().as_ptr(),
        CString::new(env::current_exe().unwrap().to_string_lossy().to_string())
          .as_ref()
          .unwrap()
          .as_ptr(),
        CString::new("").as_ref().unwrap().as_ptr(),
        CString::new("").as_ref().unwrap().as_ptr(),
        1
      );

      exit(Error::last_os_error().raw_os_error().unwrap())
    }
  }
}

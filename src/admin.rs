use std::{env, ffi::CString, io::Error, process::exit, ptr};

use winapi::um::{
  handleapi::CloseHandle,
  processthreadsapi::{GetCurrentProcess, OpenProcessToken},
  securitybaseapi::GetTokenInformation,
  shellapi::ShellExecuteA,
  winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY},
};

/// Returns true if the current process has admin rights, otherwise false.
pub fn is_elevated() -> bool {
  unsafe {
    let mut handle: HANDLE = ptr::null_mut();
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) != 0 {
      let mut elevation = TOKEN_ELEVATION::default();
      let mut ret_size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
      // The weird looking repetition of `as *mut _` is casting the reference to a c_void pointer.
      let elevated = if GetTokenInformation(
        handle,
        TokenElevation,
        &mut elevation as *mut _ as *mut _,
        ret_size,
        &mut ret_size,
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

pub fn elevate() -> crate::Result<()> {
  if !is_elevated() {
    unsafe {
      ShellExecuteA(
        ptr::null_mut(),
        CString::new("RunAs")?.as_ptr(),
        CString::new(env::current_exe()?.to_string_lossy().to_string())?.as_ptr(),
        CString::new("")?.as_ptr(),
        CString::new("")?.as_ptr(),
        1,
      );

      exit(Error::last_os_error().raw_os_error().unwrap())
    }
  }

  Ok(())
}

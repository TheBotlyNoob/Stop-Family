use std::{
  ffi::{CStr, OsStr},
  fs::File,
  path::Path
};
use winapi::um::winbase::GetUserNameA;
use windows_permissions::{
  constants::{SeObjectType, SecurityInformation},
  structures::{SecurityDescriptor, Sid},
  wrappers::{GetSecurityInfo, LookupAccountName, SetSecurityInfo},
  LocalBox
};

pub fn become_owner(file_path: &Path) -> LocalBox<SecurityDescriptor> {
  let (sid, _, _) = LookupAccountName(Option::<&OsStr>::None, unsafe {
    let buf = [0i8; 1024];
    let mut size = 0;

    GetUserNameA(buf[0] as *mut i8, &mut size);

    CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string()
  })
  .unwrap();

  set_owner(file_path, &sid)
}

pub fn set_owner(file_path: &Path, sid: &Sid) -> LocalBox<SecurityDescriptor> {
  let mut file = File::open(file_path).unwrap();

  let security_info = GetSecurityInfo(
    &file,
    SeObjectType::SE_FILE_OBJECT,
    SecurityInformation::Owner
  )
  .unwrap();

  SetSecurityInfo(
    &mut file,
    SeObjectType::SE_FILE_OBJECT,
    SecurityInformation::Owner,
    Some(sid),
    None,
    None,
    None
  )
  .unwrap();

  security_info
}

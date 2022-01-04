use std::{
  ffi::{CStr, CString, OsStr},
  fs::File,
  io::Error,
  path::Path,
  ptr
};
use winapi::um::{
  processthreadsapi::{GetCurrentProcess, OpenProcessToken},
  securitybaseapi::AdjustTokenPrivileges,
  winbase::{GetUserNameA, LookupPrivilegeValueA},
  winnt::{
    LUID, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES
  }
};
use windows_permissions::{
  constants::{SeObjectType, SecurityInformation},
  structures::{SecurityDescriptor, Sid},
  wrappers::{GetSecurityInfo, LookupAccountName, SetSecurityInfo},
  LocalBox
};

fn enable_privilege(privilege: impl AsRef<str>) -> Result<(), Error> {
  unsafe {
    let privilege = CString::new(privilege.as_ref()).unwrap();
    let mut token = ptr::null_mut();
    let mut luid = LUID::default();

    if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &mut token) == 0
      && LookupPrivilegeValueA(ptr::null_mut(), privilege.as_ptr(), &mut luid) == 0
    {
      return Err(Error::last_os_error());
    }

    let mut token_privileges = TOKEN_PRIVILEGES {
      PrivilegeCount: 1,
      Privileges: [LUID_AND_ATTRIBUTES {
        Luid: luid,
        Attributes: SE_PRIVILEGE_ENABLED
      }]
    };

    AdjustTokenPrivileges(
      token,
      0,
      &mut token_privileges,
      0,
      ptr::null_mut(),
      ptr::null_mut()
    );
  }

  Ok(())
}

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

  enable_privilege("SeTakeOwnershipPrivilege").unwrap();
  enable_privilege("SeSecurityPrivilege").unwrap();
  enable_privilege("SeBackupPrivilege").unwrap();
  enable_privilege("SeRestorePrivilhhjkkhjege").unwrap();

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

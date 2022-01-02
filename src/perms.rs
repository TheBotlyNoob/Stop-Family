use std::{
  ffi::{CStr, OsStr},
  fs::File,
  mem,
  ops::{Deref, DerefMut},
  path::PathBuf
};
use winapi::um::winbase::GetUserNameA;
use windows_permissions::{
  constants::{SeObjectType, SecurityInformation},
  structures::Sid,
  wrappers::{GetSecurityInfo, LookupAccountName, SetSecurityInfo}
};

pub fn become_owner(mut file_path: &PathBuf) -> OwnedFile {
  let mut file = File::open(file_path).unwrap();

  println!("{}", file_path.display());

  let (sid, _, _) = LookupAccountName(Option::<&OsStr>::None, unsafe {
    let buf = [0i8; 1024];
    let mut size = 0;

    GetUserNameA(buf[0] as *mut i8, &mut size);

    CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string()
  })
  .unwrap();

  let previous_security_info = GetSecurityInfo(
    &file,
    SeObjectType::SE_FILE_OBJECT,
    SecurityInformation::Owner
  )
  .unwrap();

  SetSecurityInfo(
    &mut file,
    SeObjectType::SE_FILE_OBJECT,
    SecurityInformation::Owner,
    Some(&sid),
    None,
    None,
    None
  )
  .unwrap();

  OwnedFile {
    file,
    previous_owner: Sid {
      _opaque: unsafe { mem::transmute::<&Sid, &_Sid>(previous_security_info.owner().unwrap()) }
        ._opaque
    }
  }
}

pub struct OwnedFile {
  file: File,
  previous_owner: Sid
}

struct _Sid {
  pub _opaque: [u8; 0]
}

impl Drop for OwnedFile {
  fn drop(&mut self) {
    SetSecurityInfo(
      &mut self.file,
      SeObjectType::SE_FILE_OBJECT,
      SecurityInformation::Owner,
      Some(&self.previous_owner),
      None,
      None,
      None
    )
    .unwrap();
  }
}

impl Deref for OwnedFile {
  type Target = File;

  fn deref(&self) -> &Self::Target {
    &self.file
  }
}

impl DerefMut for OwnedFile {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.file
  }
}

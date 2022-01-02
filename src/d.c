void main(void) {
  HANDLE token;
  char * filename = "somefile.txt";
  char * newuser = "someuser";
  DWORD len;
  PSECURITY_DESCRIPTOR security = NULL;
  PSID sidPtr = NULL;
  int retValue = 1;

  // Get the privileges you need
  if (OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &token)) {
    SetPrivilege(token, "SeTakeOwnershipPrivilege", 1);
    SetPrivilege(token, "SeSecurityPrivilege", 1);
    SetPrivilege(token, "SeBackupPrivilege", 1);
    SetPrivilege(token, "SeRestorePrivilege", 1);
  } else retValue = 0;

  // Create the security descriptor
  if (retValue) {
    GetFileSecurity(filename, OWNER_SECURITY_INFORMATION, security, 0, & len);
    security = (PSECURITY_DESCRIPTOR) malloc(len);
    if (!InitializeSecurityDescriptor(security,
        SECURITY_DESCRIPTOR_REVISION))
      retValue = 0;
  }

  // Get the sid for the username
  if (retValue) {
    
    char domainbuf[4096];
    DWORD sidSize = 0;
    DWORD bufSize = 4096;
    SID_NAME_USE sidUse;
    LookupAccountName(NULL, newuser, sidPtr, & sidSize, domainbuf, & bufSize, & sidUse);
    sid = (PSID) malloc(sidSize);
    if (!LookupAccountName(NULL, string, (PSID) sid, & sidSize, domainbuf, & bufSize, & sidUse))
      retValue = 0;
  }

  // Set the sid to be the new owner
  if (retValue && !SetSecurityDescriptorOwner(security, sidPtr, 0))
    retValue = 0;

  // Save the security descriptor
  if (retValue)
    retValue = SetFileSecurity(filename, OWNER_SECURITY_INFORMATION, security);
  if (security) free(security);
  if (sid) free(sid);
  return retValue;

};

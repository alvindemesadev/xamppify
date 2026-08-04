use serde::Serialize;

const MYSQL_TARGET: &str = "Xamppify/MySQL";
const ERROR_NOT_FOUND: u32 = 1168;

#[derive(Debug, Clone, Serialize)]
pub struct MysqlCredentials {
    pub user: String,
    pub password: String,
}

#[cfg(windows)]
mod native {
    use super::{MysqlCredentials, ERROR_NOT_FOUND, MYSQL_TARGET};
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
        CREDENTIALW,
    };

    fn wide(encoded: &str) -> Vec<u16> {
        encoded.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn wide_to_string(ptr: *const u16) -> String {
        if ptr.is_null() {
            return String::new();
        }
        let mut len = 0;
        while unsafe { *ptr.add(len) } != 0 {
            len += 1;
        }
        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
    }

    pub fn save(user: &str, password: &str) -> Result<(), String> {
        let target = wide(MYSQL_TARGET);
        let user_wide = wide(user);
        let mut blob: Vec<u8> = password.as_bytes().to_vec();
        blob.push(0);

        let credential = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target.as_ptr() as *mut u16,
            Comment: std::ptr::null_mut(),
            LastWritten: windows_sys::Win32::Foundation::FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: blob.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: std::ptr::null_mut(),
            TargetAlias: std::ptr::null_mut(),
            UserName: user_wide.as_ptr() as *mut u16,
        };

        let ok = unsafe { CredWriteW(&credential, 0) };
        if ok == 0 {
            return Err(format!(
                "Failed to save credentials (error {})",
                unsafe { GetLastError() }
            ));
        }
        Ok(())
    }

    pub fn get() -> Result<Option<MysqlCredentials>, String> {
        let target = wide(MYSQL_TARGET);
        let mut credential: *mut CREDENTIALW = std::ptr::null_mut();
        let ok = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
        if ok == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_NOT_FOUND {
                return Ok(None);
            }
            return Err(format!("Failed to read credentials (error {})", error));
        }
        if credential.is_null() {
            return Ok(None);
        }

        let cred = unsafe { &*credential };
        let user = wide_to_string(cred.UserName);
        let password = if cred.CredentialBlob.is_null() || cred.CredentialBlobSize == 0 {
            String::new()
        } else {
            let bytes = unsafe {
                std::slice::from_raw_parts(cred.CredentialBlob, cred.CredentialBlobSize as usize)
            };
            String::from_utf8_lossy(bytes)
                .trim_end_matches('\0')
                .to_string()
        };
        unsafe { CredFree(credential as *const core::ffi::c_void) };

        Ok(Some(MysqlCredentials { user, password }))
    }

    pub fn delete() -> Result<(), String> {
        let target = wide(MYSQL_TARGET);
        let ok = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };
        if ok == 0 {
            let error = unsafe { GetLastError() };
            if error != ERROR_NOT_FOUND {
                return Err(format!("Failed to delete credentials (error {})", error));
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
pub fn save_mysql_credentials(user: &str, password: &str) -> Result<(), String> {
    native::save(user, password)
}

#[cfg(not(windows))]
pub fn save_mysql_credentials(_user: &str, _password: &str) -> Result<(), String> {
    Err("Windows Credential Manager is only available on Windows".to_string())
}

#[cfg(windows)]
pub fn get_mysql_credentials() -> Result<Option<MysqlCredentials>, String> {
    native::get()
}

#[cfg(not(windows))]
pub fn get_mysql_credentials() -> Result<Option<MysqlCredentials>, String> {
    Ok(None)
}

#[cfg(windows)]
pub fn delete_mysql_credentials() -> Result<(), String> {
    native::delete()
}

#[cfg(not(windows))]
pub fn delete_mysql_credentials() -> Result<(), String> {
    Ok(())
}

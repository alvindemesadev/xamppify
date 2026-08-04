use std::path::Path;

/// Moves a file or folder to the Windows Recycle Bin instead of deleting it
/// permanently. Falls back to a normal delete on non-Windows platforms.
pub fn recycle_path(path: &Path) -> Result<(), String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("Failed to resolve item path: {e}"))?;

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::{
            SHFileOperationW, SHFILEOPSTRUCTW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION,
            FOF_NOERRORUI, FOF_SILENT, FO_DELETE,
        };

        let wide: Vec<u16> = canonical
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .chain(std::iter::once(0))
            .collect();

        let mut operation: SHFILEOPSTRUCTW = unsafe { std::mem::zeroed() };
        operation.wFunc = FO_DELETE;
        operation.pFrom = wide.as_ptr();
        operation.fFlags =
            (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT) as u16;

        let result = unsafe { SHFileOperationW(&mut operation) };
        if result != 0 {
            return Err(format!("Windows reported error {result} while moving the item"));
        }
        if operation.fAnyOperationsAborted != 0 {
            return Err("The move to the Recycle Bin was cancelled".to_string());
        }
        Ok(())
    }

    #[cfg(not(windows))]
    {
        if canonical.is_dir() {
            std::fs::remove_dir_all(&canonical)
                .map_err(|e| format!("Failed to remove folder: {e}"))
        } else {
            std::fs::remove_file(&canonical).map_err(|e| format!("Failed to remove file: {e}"))
        }
    }
}

//! Minimal clipboard access for SSH/SFTP password hand-off.
//!
//! The password never leaves the Rust process: it is written to the Windows
//! clipboard here and cleared again after a short delay.

/// Copies `text` to the system clipboard.
#[cfg(windows)]
pub fn copy_text(text: &str) -> bool {
    use windows_sys::Win32::Foundation::GlobalFree;
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows_sys::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
    };
    use windows_sys::Win32::System::Ole::CF_UNICODETEXT;

    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return false;
        }

        if EmptyClipboard() == 0 {
            CloseClipboard();
            return false;
        }

        let handle = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2);
        if handle.is_null() {
            CloseClipboard();
            return false;
        }

        let destination = GlobalLock(handle) as *mut u16;
        if destination.is_null() {
            GlobalFree(handle);
            CloseClipboard();
            return false;
        }

        std::ptr::copy_nonoverlapping(wide.as_ptr(), destination, wide.len());
        GlobalUnlock(handle);

        // On success the system owns the allocated memory.
        let result = SetClipboardData(CF_UNICODETEXT as u32, handle);
        CloseClipboard();

        if result.is_null() {
            GlobalFree(handle);
            return false;
        }

        true
    }
}

/// Clears the clipboard only when it still contains `expected`.
#[cfg(windows)]
pub fn clear_if_matches(expected: &str) -> bool {
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    };
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    use windows_sys::Win32::System::Ole::CF_UNICODETEXT;

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return false;
        }

        let mut matches = false;

        if IsClipboardFormatAvailable(CF_UNICODETEXT as u32) != 0 {
            let handle = GetClipboardData(CF_UNICODETEXT as u32);
            if !handle.is_null() {
                let pointer = GlobalLock(handle) as *const u16;
                if !pointer.is_null() {
                    let mut length = 0usize;
                    while *pointer.add(length) != 0 {
                        length += 1;
                    }

                    let text =
                        String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length));
                    matches = text.replace("\r\n", "\n") == expected.replace("\r\n", "\n");
                    GlobalUnlock(handle);
                }
            }
        }

        let cleared = matches && EmptyClipboard() != 0;
        CloseClipboard();
        cleared
    }
}

#[cfg(not(windows))]
pub fn copy_text(_: &str) -> bool {
    false
}

#[cfg(not(windows))]
pub fn clear_if_matches(_: &str) -> bool {
    false
}

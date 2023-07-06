use std::path::Path;

/// # Safety
/// ptr should be a zero-terminated string
pub unsafe fn u16_ptr_to_slice<'a>(ptr: *const u16) -> &'a [u16] {
    if ptr.is_null() {
        return &[];
    }

    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    return std::slice::from_raw_parts(ptr, len);
}

/// # Safety
/// ptr should be a zero-terminated string
pub unsafe fn u16_ptr_to_string(ptr: *const u16) -> String {
    let slice = unsafe { u16_ptr_to_slice(ptr) };
    String::from_utf16_lossy(slice)
}

pub fn str_to_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn str_to_utf16_no_trailing_zero(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[allow(dead_code)]
pub fn path_to_utf16(path: &Path) -> Vec<u16> {
    let s = path.to_string_lossy();
    str_to_utf16(&s)
}

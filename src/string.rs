#[allow(unused)]
macro_rules! gl_str {
    ($s:expr) => (
        concat!($s, "\0") as *const str as *const u8 as *const GLchar
    );
}

#[allow(unused)]
macro_rules! static_cstr {
    ($s:expr) => (
        {
            // Limit type.
            let _: &'static str = $s;
            $crate::string::actually_unsafe_cstr_from_bytes_with_nul_unchecked(
                concat!($s, "\0").as_bytes()
            )
        }
    )
}

use std::ffi::CStr;

#[doc(hidden)]
#[inline]
pub fn actually_unsafe_cstr_from_bytes_with_nul_unchecked(bytes: &'static[u8]) -> &'static CStr {
    unsafe { CStr::from_bytes_with_nul_unchecked(bytes) }
}

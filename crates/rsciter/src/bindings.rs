#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// see crates/rsciter/generate.ps1
#[allow(clippy::type_complexity)]
mod generated;

pub use generated::*;

// define _PTR type manually, as they depends on platform
pub type INT_PTR = isize;
pub type UINT_PTR = usize;
pub type LONG_PTR = isize;

pub type LPRECT = *mut RECT;
pub type LPPOINT = *mut POINT;
pub type LPSIZE = *mut SIZE;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        // TODO: if windowless HWND = LPVOID
        pub use windows::Win32::Foundation::HWND;

        pub use windows::Win32::Foundation::{WPARAM, LPARAM, POINT, RECT, LRESULT, SIZE};
        pub use windows::Win32::UI::WindowsAndMessaging::MSG;
        pub use windows::core::IUnknown;

        pub const SCITER_DLL_NAME: &str = "sciter.dll";
    }
    else if #[cfg(target_os="android")] {
        pub use HWND = isize;
        pub const SCITER_DLL_NAME: &str = "libsciter.so";
    }
    else if #[cfg(target_os="linux")] {
        pub use HWND = isize;

        pub const SCITER_DLL_NAME: &str = "libsciter.so";
    }
    else if #[cfg(target_os="macos")] {
        pub use HWND = isize;

        pub const SCITER_DLL_NAME: &str = "libsciter.dylib";
    }
}

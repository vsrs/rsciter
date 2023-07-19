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
    else {
        pub type HWND = isize;

        #[derive(Default, Clone, Copy, PartialEq, Eq)]
        #[repr(C)]
        pub struct POINT {
            pub x: i32,
            pub y: i32,
        }

        #[derive(Default, Clone, Copy, PartialEq, Eq)]
        #[repr(C)]
        pub struct SIZE {
            pub cx: i32,
            pub cy: i32,
        }

        #[derive(Default, Clone, Copy, PartialEq, Eq)]
        #[repr(C)]
        pub struct RECT {
            pub left: i32,
            pub top: i32,
            pub right: i32,
            pub bottom: i32,
        }

        #[derive(Default, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct WPARAM(pub usize);

        #[derive(Default, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct LPARAM(pub isize);

        #[derive(Default, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct LRESULT(pub isize);

        // just to make rustc happy:
        pub struct MSG;
        pub struct IUnknown;

        cfg_if::cfg_if! {
            if #[cfg(any(target_os="android", target_os="linux"))] {
                pub const SCITER_DLL_NAME: &str = "libsciter.so";
            }
            else if #[cfg(target_os="macos")] {
                pub const SCITER_DLL_NAME: &str = "libsciter.dylib";
            }
        }
    }
}

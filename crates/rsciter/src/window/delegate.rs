use super::{WindowHandle, WindowState};
use crate::bindings::*;

pub trait AsAny: 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: Sized + 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait WindowDelegate: AsAny {
    fn on_message(
        &mut self,
        window: WindowHandle,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> (bool, LRESULT) {
        let _ = window;
        let _ = msg;
        let _ = wparam;
        let _ = lparam;

        (false, Default::default())
    }
}

impl<F> WindowDelegate for F
where
    F: Fn(WindowHandle, UINT, WPARAM, LPARAM) -> (bool, LRESULT) + AsAny,
{
    fn on_message(
        &mut self,
        window: WindowHandle,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> (bool, LRESULT) {
        self(window, msg, wparam, lparam)
    }
}

pub(super) unsafe extern "C" fn window_delegate_thunk(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
    dparam: LPVOID,
    handled: *mut SBOOL,
) -> LRESULT {
    if !dparam.is_null() {
        let state_ptr = dparam as *mut WindowState;
        if let Some(delegate) = (*state_ptr).delegate() {
            let window = WindowHandle::from(hwnd);
            let (x_handled, res) = delegate.on_message(window, msg, wparam, lparam);

            if !handled.is_null() && x_handled {
                *handled = 1;
            }

            return res;
        }
    }

    Default::default()
}

use crate::{api::sapi, bindings::*, Error, Result, Value};

/// A handle to a Sciter window object.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowHandle {
    hwnd: HWND,
}

impl From<HWND> for WindowHandle {
    fn from(hwnd: HWND) -> Self {
        WindowHandle { hwnd }
    }
}

impl From<WindowHandle> for HWND {
    fn from(value: WindowHandle) -> Self {
        value.hwnd
    }
}

impl WindowHandle {
    pub fn is_valid(&self) -> bool {
        self.hwnd != HWND::default()
    }

    pub fn collapse(&self) -> Result<()> {
        self.set_state(SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_MINIMIZED)
    }

    pub fn show(&self, kind: Visibility) -> Result<()> {
        let state = match kind {
            Visibility::Normal => SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_SHOWN,
            Visibility::Maximized => SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_MAXIMIZED,
            Visibility::FullScreen => SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_FULL_SCREEN,
        };

        self.set_state(state)
    }

    pub fn hide(&self) -> Result<()> {
        self.set_state(SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_HIDDEN)
    }

    /// Close the window, scripts can reject the closure
    pub fn request_close(&self) -> Result<()> {
        self.set_state_with_arg(SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_CLOSED, false as _)
    }

    /// Close the window, scripts cannot reject the closure
    pub fn close(&self) -> Result<()> {
        self.set_state_with_arg(SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_CLOSED, true as _)
    }

    pub fn activate(&self) -> Result<()> {
        self.activate_impl(false)
    }

    pub fn bring_to_front(&self) -> Result<()> {
        self.activate_impl(true)
    }

    /// Loads HTML file.
    ///
    /// Returns `true` if the text was parsed and loaded successfully, 'false' otherwise.
    pub fn load_file(&self, path: impl AsRef<str>) -> Result<bool> {
        sapi()?.load_file(self.hwnd, path)
    }

    /// Loads an HTML document from memory.
    ///
    /// Returns `true` if the document was parsed and loaded successfully, 'false' otherwise.
    pub fn load_html(&self, html: &[u8], base_url: Option<&str>) -> Result<bool> {
        sapi()?.load_html(self.hwnd, html, base_url)
    }

    /// Posts host notifiacation. The host will get it in [`HostNotifications::on_posted_notification`]
    pub fn notify_host(
        &self,
        wparam: UINT_PTR,
        lparam: UINT_PTR,
        timeoutms: UINT,
    ) -> Result<UINT_PTR> {
        sapi()?.post_callback(self.hwnd, wparam, lparam, timeoutms)
    }

    pub fn eval(&self, script: &str) -> Result<Value> {
        sapi()?
            .eval(self.hwnd, script)
            .and_then(Self::check_error_string)
    }

    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value> {
        sapi()?
            .call(self.hwnd, name, args)
            .and_then(Self::check_error_string)
    }

    fn check_error_string(value: Value) -> Result<Value> {
        if value.is_error_string() {
            let err = value.to_string().unwrap_or_else(|_| "unknown".to_string());
            return Err(Error::ScriptError(err));
        }

        Ok(value)
    }
}

pub enum Visibility {
    Normal,
    Maximized,
    FullScreen,
}

// details
impl WindowHandle {
    fn set_state(&self, state: SCITER_WINDOW_STATE) -> Result<()> {
        self.set_state_with_arg(state, 0)
    }

    fn set_state_with_arg(&self, state: SCITER_WINDOW_STATE, arg: UINT_PTR) -> Result<()> {
        let _res = sapi()?.window_exec(
            self.hwnd,
            SCITER_WINDOW_CMD::SCITER_WINDOW_SET_STATE,
            state as _,
            arg,
        )?;
        Ok(())
    }

    fn activate_impl(&self, bring_to_front: bool) -> Result<()> {
        let _res = sapi()?.window_exec(
            self.hwnd,
            SCITER_WINDOW_CMD::SCITER_WINDOW_ACTIVATE,
            bring_to_front as _,
            0,
        )?;
        Ok(())
    }
}

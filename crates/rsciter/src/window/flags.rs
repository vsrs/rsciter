use crate::bindings::SCITER_CREATE_WINDOW_FLAGS;

pub struct WindowFlags(pub(crate) i32);

impl WindowFlags {
    /// child window only, if this flag is set all other flags ignored  
    pub const CHILD: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_CHILD.0);
    /// toplevel window, has titlebar
    pub const HAS_TITLEBAR: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_TITLEBAR.0);
    /// has resizeable frame
    pub const RESIZEABLE: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_RESIZEABLE.0);
    /// is tool window
    pub const TOOL: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_TOOL.0);
    /// has minimize / maximize buttons
    pub const HAS_CONTROLS: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_CONTROLS.0);
    /// glassy window - supports "Acrylic" on Windows and "Vibrant" on MacOS.
    pub const GLASSY: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_GLASSY.0);
    /// transparent window ( e.g. WS_EX_LAYERED on Windows )
    pub const ALPHA: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_ALPHA.0);
    /// main window of the app, will terminate the app on close
    pub const MAIN: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_MAIN.0);
    /// the window is created as topmost window.
    pub const POPUP: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_POPUP.0);
    /// make this window inspector ready
    pub const ENABLE_DEBUG: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_ENABLE_DEBUG.0);
    // it has its own script VM
    pub const OWNS_VM: Self = Self(SCITER_CREATE_WINDOW_FLAGS::SW_OWNS_VM.0);
}

impl ::std::ops::BitOr<WindowFlags> for WindowFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl ::std::ops::BitOrAssign for WindowFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: WindowFlags) {
        self.0 |= rhs.0;
    }
}

impl ::std::ops::BitAnd<WindowFlags> for WindowFlags {
    type Output = Self;
    #[inline]
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl ::std::ops::BitAndAssign for WindowFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl WindowFlags {
    pub fn remove(&mut self, flags: WindowFlags) -> &mut Self {
        self.0 &= !flags.0;
        self
    }
}

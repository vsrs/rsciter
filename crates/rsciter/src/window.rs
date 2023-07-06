use std::{fmt::Debug, ops::Deref, pin::Pin};

use crate::{EventHandler, Result};

mod builder;
mod delegate;
mod flags;
mod handle;
mod host;

pub use builder::*;
pub use delegate::*;
pub use flags::*;
pub use handle::*;
pub use host::*;

pub struct Window {
    handle: WindowHandle,
    state: Pin<Box<WindowState>>,
}

impl Deref for Window {
    type Target = WindowHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("handle", &self.handle)
            .finish()
    }
}

impl Window {
    /// Constructs a new Sciter main window with default flags and position.
    ///
    /// `Main` means that the window will terminate the app on close.
    pub fn new() -> Result<Window> {
        WindowBuilder::new().build_main()
    }

    /// Constructs a new Sciter window with default flags and position.
    pub fn new_secondary() -> Result<Window> {
        WindowBuilder::new().build_secondary()
    }

    /// Returns a new [`WindowBuilder`] with default falgs set, including [`WindowFlags::MAIN`].
    pub fn builder<'b>() -> WindowBuilder<'b> {
        WindowBuilder::new()
    }

    pub fn handle(&self) -> WindowHandle {
        self.handle
    }

    pub fn has_window_delegate(&self) -> bool {
        self.state.as_ref().delegate.is_some()
    }

    pub fn has_host(&self) -> bool {
        self.state.as_ref().host.is_some()
    }

    pub fn has_event_handler(&self) -> bool {
        self.state.as_ref().event_handler.is_some()
    }

    /// Get access to the [`WindowDelegate`] trait object if any.
    pub fn with_window_delegate<T: WindowDelegate>(&self, f: impl FnOnce(&T)) {
        if let Some(dyn_delegate) = self.state.as_ref().delegate.as_ref() {
            if let Some(delegate) = dyn_delegate.as_any().downcast_ref::<T>() {
                f(delegate);
            }
        }
    }

    pub fn with_window_delegate_mut<T: WindowDelegate>(&mut self, f: impl FnOnce(&T)) {
        if let Some(dyn_delegate) = self.state.as_mut().delegate.as_mut() {
            if let Some(delegate) = dyn_delegate.as_any_mut().downcast_mut::<T>() {
                f(delegate);
            }
        }
    }

    /// Get access to the [`HostNotifications`] trait object if any.
    pub fn with_host<T: HostNotifications>(&self, f: impl FnOnce(&T)) {
        if let Some(dyn_host) = self.state.as_ref().host.as_ref() {
            if let Some(host) = dyn_host.as_any().downcast_ref::<T>() {
                f(host);
            }
        }
    }

    pub fn with_host_mut<T: HostNotifications>(&mut self, f: impl FnOnce(&T)) {
        if let Some(dyn_host) = self.state.as_mut().host.as_mut() {
            if let Some(host) = dyn_host.as_any_mut().downcast_mut::<T>() {
                f(host);
            }
        }
    }

    /// Get access to the [`EventHandler`] trait object if any.
    pub fn with_event_handler<T: EventHandler>(&self, f: impl FnOnce(&T)) {
        if let Some(dyn_handler) = self.state.as_ref().event_handler.as_ref() {
            if let Some(handler) = dyn_handler.as_any().downcast_ref::<T>() {
                f(handler);
            }
        }
    }

    pub fn with_event_handler_mut<T: EventHandler>(&mut self, f: impl FnOnce(&T)) {
        if let Some(dyn_handler) = self.state.as_mut().event_handler.as_mut() {
            if let Some(handler) = dyn_handler.as_any_mut().downcast_mut::<T>() {
                f(handler);
            }
        }
    }
}

pub(crate) struct WindowState {
    delegate: Option<Box<dyn WindowDelegate>>,
    host: Option<Box<dyn HostNotifications>>,
    event_handler: Option<Box<dyn EventHandler>>,
}

impl WindowState {
    pub(crate) fn delegate(&mut self) -> Option<&mut dyn WindowDelegate> {
        self.delegate.as_mut().map(|it| it.as_mut())
    }

    pub(crate) fn host(&mut self) -> Option<&mut dyn HostNotifications> {
        self.host.as_mut().map(|it| it.as_mut())
    }

    pub(crate) fn event_handler(&mut self) -> Option<&mut dyn EventHandler> {
        self.event_handler.as_mut().map(|it| it.as_mut())
    }
}

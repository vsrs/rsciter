use std::{collections::HashMap, pin::Pin};

use super::{HostNotifications, Window, WindowDelegate, WindowFlags, WindowHandle, WindowState};
use crate::{
    api::sapi, bindings::*, ArchiveData, DefaultEventHandler, DefaultHost, EventHandler, Result,
    XFunction, XFunctionProvider,
};

// Some rust black magic to disallow
// -  `with_html` and `with_file` simultaneous calls on the same builder object
// -  calling `with_archive`, etc after setting custom host (calling `with_custom_host`)

/// A builder type for creating new windows.
pub struct WindowBuilder<'b, const HOST: u8 = HOST_NONE, const INITIAL_PAGE: u8 = INITIAL_PAGE_NONE>
{
    common: Common,
    initial_page: InitialPage<'b>,
    host: Host,
}

impl Default for WindowBuilder<'_, HOST_NONE, INITIAL_PAGE_NONE> {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowBuilder<'_, HOST_NONE, INITIAL_PAGE_NONE> {
    /// Creates a new `WindowBuilder`
    pub fn new() -> Self {
        let flags = WindowFlags::HAS_TITLEBAR | WindowFlags::HAS_CONTROLS | WindowFlags::RESIZEABLE;

        Self {
            common: Common {
                flags,
                frame: None,
                parent: None,
                window_delegate: None,
                event_handler: None,
            },
            initial_page: InitialPage::None,
            host: Host::None,
        }
    }
}

// common case
impl<'b, const ANY_HOST: u8, const ANY_INITIAL_PAGE: u8>
    WindowBuilder<'b, ANY_HOST, ANY_INITIAL_PAGE>
{
    /// Sets the window's initial [`WindowFlags`].
    ///
    /// Be careful, this call overwrites all default flags set in [`WindowBuilder::new`]
    pub fn with_flags(mut self, flags: WindowFlags) -> Self {
        self.common.flags = flags;
        self
    }

    pub fn with_flags_or(mut self, flags: WindowFlags) -> Self {
        self.common.flags |= flags;
        self
    }

    /// Set the window's initial size and position.
    pub fn with_frame(mut self, frame: RECT) -> Self {
        self.common.frame = Some(frame);
        self
    }

    pub fn with_window_delegate(mut self, delegate: impl WindowDelegate) -> Self {
        self.common.window_delegate = Some(Box::new(delegate));
        self
    }

    pub fn with_event_handler(mut self, handler: impl EventHandler) -> Self {
        self.common.event_handler = Some(Box::new(handler));
        self
    }

    /// Attempt to construct the main Sciter window.
    /// Explicitly sets `WindowFlags::MAIN` flag
    pub fn build_main(mut self) -> Result<Window> {
        self.common.flags |= WindowFlags::MAIN;
        self.build()
    }

    /// Attempt to construct a secondary Sciter window.
    /// Explicitly removes `WindowFlags::MAIN` flag
    pub fn build_secondary(mut self) -> Result<Window> {
        self.common.flags.remove(WindowFlags::MAIN);
        self.build()
    }

    /// Attempt to construct a Sciter window using existing [WindowFlags]
    fn build(self) -> Result<Window> {
        let api = sapi()?;

        let has_window_delegate = self.common.window_delegate.is_some();
        let host_info = self.host.get()?;
        let event_handler = match (host_info.event_handler, self.common.event_handler) {
            (None, None) => None,
            (None, Some(user)) => Some(user),
            (Some(default), None) => {
                let handler: Box<dyn EventHandler> = Box::new(default);
                Some(handler)
            }
            (Some(mut default), Some(user)) => {
                default.set_custom_event_handler(user);
                let handler: Box<dyn EventHandler> = Box::new(default);
                Some(handler)
            }
        };

        let state = WindowState {
            delegate: self.common.window_delegate,
            host: host_info.host,
            event_handler,
        };
        let mut pinned = Box::pin(state);
        let state_ptr = unsafe { Pin::get_unchecked_mut(pinned.as_mut()) as *mut WindowState };

        let flags = self.common.flags.0 as UINT;
        let hwnd = if has_window_delegate {
            api.create_window(
                flags,
                self.common.frame,
                self.common.parent,
                Some(super::window_delegate_thunk),
                state_ptr as _,
            )?
        } else {
            api.create_window(flags, self.common.frame, self.common.parent, None, 0 as _)?
        };

        let window = Window {
            handle: WindowHandle::from(hwnd),
            state: pinned,
        };

        if window.state.host.is_some() {
            api.set_callback(
                window.handle().into(),
                Some(super::host_thunk),
                state_ptr as _,
            )?;
        }

        if window.state.event_handler.is_some() {
            api.window_attach_event_handler(
                window.handle.into(),
                Some(crate::element_proc_thunk),
                state_ptr as _,
                EVENT_GROUPS::HANDLE_ALL,
            )?;
        }

        match self.initial_page {
            InitialPage::None => Ok(window),
            InitialPage::Html { html, base_url } => {
                let _res = window.load_html(html, base_url)?;
                // TODO: error?
                Ok(window)
            }
            InitialPage::File(file) => {
                let _res = window.load_file(file)?;
                // TODO: error?
                Ok(window)
            }
        }
    }
}

// initial page not set
impl<'b, const ANY_HOST: u8> WindowBuilder<'b, ANY_HOST, INITIAL_PAGE_NONE> {
    pub fn with_html(self, html: &'b [u8]) -> WindowBuilder<'b, ANY_HOST, INITIAL_PAGE_HTML> {
        WindowBuilder {
            common: self.common,
            initial_page: InitialPage::Html {
                html,
                base_url: None,
            },
            host: self.host,
        }
    }

    pub fn with_file(self, file: &'b str) -> WindowBuilder<'b, ANY_HOST, INITIAL_PAGE_FILE> {
        WindowBuilder {
            common: self.common,
            initial_page: InitialPage::File(file),
            host: self.host,
        }
    }
}

// initial page is HTML, we may set base url
impl<'b, const ANY_HOST: u8> WindowBuilder<'b, ANY_HOST, INITIAL_PAGE_HTML> {
    pub fn with_base_url(mut self, url: &'b str) -> Self {
        match &mut self.initial_page {
            InitialPage::Html { base_url, .. } => *base_url = Some(url),
            _ => unreachable!(),
        }
        self
    }
}

// host is unset
impl<'b, const ANY_INITIAL_PAGE: u8> WindowBuilder<'b, HOST_NONE, ANY_INITIAL_PAGE> {
    // inherent associated types are unstable
    // https://github.com/rust-lang/rust/issues/8995
    //
    // type WithDefaultHost = Test<'b, HOST_TYPE_DEFAULT, INITIAL_TYPE>;

    pub fn with_archive_static(
        self,
        data: &'static [u8],
    ) -> WindowBuilder<'b, HOST_DEFAULT, ANY_INITIAL_PAGE> {
        self.with_default_host().with_archive_static(data)
    }

    pub fn with_archive(self, data: Vec<u8>) -> WindowBuilder<'b, HOST_DEFAULT, ANY_INITIAL_PAGE> {
        self.with_default_host().with_archive(data)
    }

    pub fn with_host(
        self,
        host: impl HostNotifications,
    ) -> WindowBuilder<'b, HOST_CUSTOM, ANY_INITIAL_PAGE> {
        WindowBuilder {
            common: self.common,
            initial_page: self.initial_page,
            host: Host::Custom(Box::new(host)),
        }
    }

    pub fn with_default_host(self) -> WindowBuilder<'b, HOST_DEFAULT, ANY_INITIAL_PAGE> {
        WindowBuilder {
            common: self.common,
            initial_page: self.initial_page,
            host: Host::Default {
                archive_data: None,
                archive_uri: None,
                functions: Default::default(),
                modules: Default::default(),
            },
        }
    }

    pub fn with_function(
        self,
        name: impl AsRef<str>,
        func: impl XFunction,
    ) -> WindowBuilder<'b, HOST_DEFAULT, ANY_INITIAL_PAGE> {
        self.with_default_host().with_function(name, func)
    }

    pub fn with_module(
        self,
        provider: impl XFunctionProvider,
    ) -> WindowBuilder<'b, HOST_DEFAULT, ANY_INITIAL_PAGE> {
        self.with_default_host().with_module(provider)
    }
}

impl<'b, const ANY_INITIAL_PAGE: u8> WindowBuilder<'b, HOST_DEFAULT, ANY_INITIAL_PAGE> {
    pub fn with_archive_static(mut self, data: &'static [u8]) -> Self {
        match &mut self.host {
            Host::Default { archive_data, .. } => *archive_data = Some(ArchiveData::Static(data)),
            _ => unreachable!(),
        }
        self
    }

    pub fn with_archive(mut self, data: Vec<u8>) -> Self {
        match &mut self.host {
            Host::Default { archive_data, .. } => *archive_data = Some(ArchiveData::Heap(data)),
            _ => unreachable!(),
        }
        self
    }

    pub fn with_archive_uri(mut self, uri: String) -> Self {
        match &mut self.host {
            Host::Default { archive_uri, .. } => *archive_uri = Some(uri),
            _ => unreachable!(),
        }
        self
    }

    pub fn with_function(mut self, name: impl AsRef<str>, func: impl XFunction) -> Self {
        match &mut self.host {
            Host::Default { functions, .. } => {
                functions.insert(name.as_ref().to_string(), Box::new(func));
            }
            _ => unreachable!(),
        }
        self
    }

    pub fn with_module(mut self, provider: impl XFunctionProvider) -> Self {
        match &mut self.host {
            Host::Default { modules, .. } => {
                modules.push(Box::new(provider));
            }
            _ => unreachable!(),
        }
        self
    }
}

const HOST_NONE: u8 = 0;
const HOST_DEFAULT: u8 = 1;
const HOST_CUSTOM: u8 = 2;

const INITIAL_PAGE_NONE: u8 = 0;
const INITIAL_PAGE_HTML: u8 = 1;
const INITIAL_PAGE_FILE: u8 = 2;

struct Common {
    flags: WindowFlags,
    frame: Option<RECT>,
    parent: Option<HWND>,
    window_delegate: Option<Box<dyn WindowDelegate>>,
    event_handler: Option<Box<dyn EventHandler>>,
}

enum InitialPage<'b> {
    None,
    Html {
        html: &'b [u8],
        base_url: Option<&'b str>,
    },
    File(&'b str),
}

enum Host {
    None,
    Default {
        archive_data: Option<ArchiveData>,
        archive_uri: Option<String>,
        functions: HashMap<String, Box<dyn XFunction>>,
        modules: Vec<Box<dyn XFunctionProvider>>,
    },
    Custom(Box<dyn HostNotifications>),
}

struct HostInfo {
    host: Option<Box<dyn HostNotifications>>,
    event_handler: Option<DefaultEventHandler>,
}

impl HostInfo {
    fn new(
        host: Box<dyn HostNotifications>,
        functions: HashMap<String, Box<dyn XFunction>>,
        modules: Vec<Box<dyn XFunctionProvider>>,
    ) -> Self {
        let event_handler = if !functions.is_empty() || !modules.is_empty() {
            Some(DefaultEventHandler::new(functions, modules))
        } else {
            None
        };

        Self {
            host: Some(host),
            event_handler,
        }
    }

    fn with_host(host: Box<dyn HostNotifications>) -> Self {
        Self {
            host: Some(host),
            event_handler: None,
        }
    }

    pub fn none() -> Self {
        Self {
            host: None,
            event_handler: None,
        }
    }
}

impl Host {
    fn get(self) -> Result<HostInfo> {
        match self {
            Host::None => Ok(HostInfo::none()),
            Host::Default {
                archive_data,
                archive_uri,
                functions,
                modules,
            } => {
                let mut host = archive_uri
                    .map(DefaultHost::with_archive_uri)
                    .unwrap_or_else(DefaultHost::new);

                if let Some(archive_data) = archive_data {
                    host.set_archive(archive_data)?;
                }

                Ok(HostInfo::new(Box::new(host), functions, modules))
            }
            Host::Custom(host) => Ok(HostInfo::with_host(host)),
        }
    }
}

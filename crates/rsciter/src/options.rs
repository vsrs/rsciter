use crate::{
    api::sapi,
    bindings::{GFX_LAYER, SCITER_RT_OPTIONS, SCRIPT_RUNTIME_FEATURES, UINT_PTR},
    Result,
};

pub type GfxLayer = GFX_LAYER;

pub fn set_option(option: RuntimeOption) -> Result<bool> {
    let (option, value) = match option {
        RuntimeOption::SmoothScroll(v) => (SCITER_RT_OPTIONS::SCITER_SMOOTH_SCROLL, v as UINT_PTR),
        RuntimeOption::ConnectionTimeout(v) => {
            (SCITER_RT_OPTIONS::SCITER_CONNECTION_TIMEOUT, v as UINT_PTR)
        }
        RuntimeOption::HttpsErrorAction(v) => {
            (SCITER_RT_OPTIONS::SCITER_HTTPS_ERROR, v as UINT_PTR)
        }
        RuntimeOption::FontSmoothing(v) => {
            (SCITER_RT_OPTIONS::SCITER_FONT_SMOOTHING, v as UINT_PTR)
        }
        RuntimeOption::TransparentWindow(v) => {
            (SCITER_RT_OPTIONS::SCITER_TRANSPARENT_WINDOW, v as UINT_PTR)
        }
        RuntimeOption::ScriptFeatures(v) => (
            SCITER_RT_OPTIONS::SCITER_SET_SCRIPT_RUNTIME_FEATURES,
            v.0 as UINT_PTR,
        ),
        RuntimeOption::GfxLayer(v) => (SCITER_RT_OPTIONS::SCITER_SET_GFX_LAYER, v as UINT_PTR),
        RuntimeOption::DebugMode(v) => (SCITER_RT_OPTIONS::SCITER_SET_DEBUG_MODE, v as UINT_PTR),
        RuntimeOption::UxTheming(v) => (SCITER_RT_OPTIONS::SCITER_SET_UX_THEMING, v as UINT_PTR),
        RuntimeOption::AlphaWindow(v) => (SCITER_RT_OPTIONS::SCITER_ALPHA_WINDOW, v as UINT_PTR),
        RuntimeOption::InitScript(v) => (
            SCITER_RT_OPTIONS::SCITER_SET_INIT_SCRIPT,
            v.as_bytes().as_ptr() as UINT_PTR,
        ),
        RuntimeOption::MainWindow(v) => (SCITER_RT_OPTIONS::SCITER_SET_MAIN_WINDOW, v as UINT_PTR),
        RuntimeOption::MaxHttpDataSize(v) => (
            SCITER_RT_OPTIONS::SCITER_SET_MAX_HTTP_DATA_LENGTH,
            v as UINT_PTR,
        ),
        RuntimeOption::PxAsDip(v) => (SCITER_RT_OPTIONS::SCITER_SET_PX_AS_DIP, v as UINT_PTR),
    };

    sapi()?.set_option(None, option, value)
}

pub enum RuntimeOption {
    /// true - enabled, default
    SmoothScroll(bool),
    /// In milliseconds
    ConnectionTimeout(usize),
    HttpsErrorAction(HttpsErrorAction),
    FontSmoothing(FontSmoothing),
    TransparentWindow(bool),
    // GpuBlacklist, not used: https://sciter.com/forums/topic/how-to-use-the-gpu-blacklist/#post-59338
    ScriptFeatures(ScriptFeatures),
    GfxLayer(GfxLayer),
    DebugMode(bool),
    /// true - the engine will use "unisex" theme that is common for all platforms.
    UxTheming(bool),
    /// Use per pixel alpha (e.g. `WS_EX_LAYERED`/`UpdateLayeredWindow()` window)
    AlphaWindow(bool),
    /// UTF-8 encoded script source to be loaded into each view before any other script execution.
    /// Should be zero-terminated!!!
    InitScript(String),
    /// Main window will destroy all other dependent windows on close.
    MainWindow(bool),
    /// In megabytes
    MaxHttpDataSize(usize),
    /// true - 1px in CSS is treated as 1dip, false (default) - 1px is a physical pixel.  
    PxAsDip(bool),
}

pub enum HttpsErrorAction {
    Drop = 0,
    BuiltinDialog = 1,
    Accept = 2,
}

pub enum FontSmoothing {
    SystemDefault = 0,
    No = 1,
    Std = 2,
    ClearType = 3,
}

pub struct ScriptFeatures(pub(crate) i32);

impl ScriptFeatures {
    pub const ALLOW_FILE_IO: Self = Self(SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO.0);
    pub const ALLOW_SOCKET_IO: Self = Self(SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO.0);
    pub const ALLOW_EVAL: Self = Self(SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL.0);
    pub const ALLOW_SYSINFO: Self = Self(SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO.0);

    pub const ALLOW_ALL_IO: Self = Self(Self::ALLOW_FILE_IO.0 | Self::ALLOW_SOCKET_IO.0);
    pub const ALLOW_ALL: Self =
        Self(Self::ALLOW_ALL_IO.0 | Self::ALLOW_EVAL.0 | Self::ALLOW_SYSINFO.0);
}

impl ::std::ops::BitOr<ScriptFeatures> for ScriptFeatures {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl ::std::ops::BitOrAssign for ScriptFeatures {
    #[inline]
    fn bitor_assign(&mut self, rhs: ScriptFeatures) {
        self.0 |= rhs.0;
    }
}

impl ::std::ops::BitAnd<ScriptFeatures> for ScriptFeatures {
    type Output = Self;
    #[inline]
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl ::std::ops::BitAndAssign for ScriptFeatures {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

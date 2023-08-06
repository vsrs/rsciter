use crate::{api::*, bindings::*, Result};

pub trait DebugNotifications {
    fn output(&self, subsystem: DebugSubsystem, severity: DebugSeverity, text: String);
}

#[derive(Debug)]
pub enum DebugSubsystem {
    Dom,
    Css,
    Csss,
    Script,
    Unknown(i32),
}

impl From<OUTPUT_SUBSYTEMS> for DebugSubsystem {
    fn from(value: OUTPUT_SUBSYTEMS) -> Self {
        match value {
            OUTPUT_SUBSYTEMS::OT_DOM => Self::Dom,
            OUTPUT_SUBSYTEMS::OT_CSS => Self::Css,
            OUTPUT_SUBSYTEMS::OT_CSSS => Self::Csss,
            OUTPUT_SUBSYTEMS::OT_TIS => Self::Script,
            v => Self::Unknown(v.0),
        }
    }
}

#[derive(Debug)]
pub enum DebugSeverity {
    Info,
    Warning,
    Error,
    Unknown(i32),
}

impl From<OUTPUT_SEVERITY> for DebugSeverity {
    fn from(value: OUTPUT_SEVERITY) -> Self {
        match value {
            OUTPUT_SEVERITY::OS_INFO => Self::Info,
            OUTPUT_SEVERITY::OS_WARNING => Self::Warning,
            OUTPUT_SEVERITY::OS_ERROR => Self::Error,
            x => Self::Unknown(x.0),
        }
    }
}

#[must_use = "rsciter::DebugOutputGuard value should be alive until the end, or there will be no debug output!"]
pub struct DebugOutputGuard {
    #[allow(dead_code)]
    handler: Box<Inner>,
}

struct Inner(Box<dyn DebugNotifications>);

impl Drop for DebugOutputGuard {
    fn drop(&mut self) {
        if let Ok(api) = sapi() {
            let _ = api.setup_debug_output(None, std::ptr::null_mut(), None);
        }
        // now it's safe to drop handler
    }
}

impl<F> DebugNotifications for F
where
    F: Fn(DebugSubsystem, DebugSeverity, String),
{
    fn output(&self, subsystem: DebugSubsystem, severity: DebugSeverity, text: String) {
        self(subsystem, severity, text)
    }
}

pub fn setup_debug_output(output: impl DebugNotifications + 'static) -> Result<DebugOutputGuard> {
    let mut boxed = Box::new(Inner(Box::new(output)));
    let ptr = boxed.as_mut() as *const Inner as *mut _;

    sapi()?.setup_debug_output(None, ptr, Some(debug_thunk))?;

    Ok(DebugOutputGuard { handler: boxed })
}

unsafe extern "C" fn debug_thunk(
    param: LPVOID,
    subsystem: UINT,
    severity: UINT,
    text: LPCWSTR,
    text_length: UINT,
) {
    if param.is_null() {
        return;
    }

    let inner = &*(param as *const Inner);
    let data = std::slice::from_raw_parts(text, text_length as usize);
    let subsystem = OUTPUT_SUBSYTEMS(subsystem as i32).into();
    let severity = OUTPUT_SEVERITY(severity as i32).into();
    inner
        .0
        .output(subsystem, severity, String::from_utf16_lossy(data));
}

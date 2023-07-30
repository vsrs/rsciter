use std::pin::Pin;

use crate::{api::*, bindings::*, Result};

pub trait DebugNotifications {
    fn output(&self, subsystem: OUTPUT_SUBSYTEMS, severity: OUTPUT_SEVERITY, text: u32);
}

pub struct DebugOutputGuard {
    handler: Box<dyn DebugNotifications>,
}

impl<F> DebugNotifications for F
where
    F: Fn(OUTPUT_SUBSYTEMS, OUTPUT_SEVERITY, u32),
{
    fn output(&self, subsystem: OUTPUT_SUBSYTEMS, severity: OUTPUT_SEVERITY, text: u32) {
        self(subsystem, severity, text)
    }
}

pub fn setup_debug_output(output: impl DebugNotifications + 'static) -> Result<DebugOutputGuard> {
    let mut boxed = Box::new(output);
    let ptr = boxed.as_mut() as *const dyn DebugNotifications;

    let d = unsafe { &*ptr };
    d.output(OUTPUT_SUBSYTEMS::OT_CSS, OUTPUT_SEVERITY::OS_INFO, 1);

    Ok(DebugOutputGuard { handler: boxed })
}

unsafe extern "C" fn debug_thunk(
    param: LPVOID,
    subsystem: UINT,
    severity: UINT,
    text: LPCWSTR,
    text_length: UINT,
) {
}

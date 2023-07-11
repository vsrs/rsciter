use std::{borrow::Cow, ffi::CStr};

use crate::{bindings::*, utf, AsAny};

use super::WindowState;

mod default;
mod default_event_handler;

pub use default::*;
pub use default_event_handler::*;
pub type LoadDataResult = SC_LOAD_DATA_RETURN_CODES;

pub trait HostNotifications: AsAny {
    fn on_load_data(&mut self, data: &LoadData) -> (LoadDataResult, Option<&[u8]>) {
        let _ = data;

        (LoadDataResult::LOAD_OK, None)
    }

    fn on_data_loaded(&mut self, data: &DataLoaded) {
        let _ = data;
    }

    fn on_engine_destroyed(&mut self, hwnd: HWND) {
        let _ = hwnd;
    }

    fn on_graphics_critical_failure(&mut self, hwnd: HWND) {
        let _ = hwnd;
    }

    fn on_unknown(&mut self, hwnd: HWND, code: u32) {
        let _ = hwnd;
        let _ = code;
    }

    /// https://sciter.com/forums/topic/sciterpostcallback-not-working-in-osx-4-1-0-5687/
    /// Not sure what SCN_POSTED_NOTIFICATION::lreturn member means, so returning None is a good strategy.
    fn on_posted_notification(
        &mut self,
        hwnd: HWND,
        wparam: UINT_PTR,
        lparam: UINT_PTR,
    ) -> Option<UINT_PTR> {
        let _ = hwnd;
        let _ = wparam;
        let _ = lparam;

        None
    }

    fn on_attach_behavior(
        &mut self,
        request: AttachBehaviorRequest,
    ) -> Option<AttachBehaviorResponse> {
        let _ = request;

        None
    }
}

#[repr(transparent)]
pub struct LoadData<'a>(&'a SCN_LOAD_DATA);

impl<'a> LoadData<'a> {
    pub fn hwnd(&self) -> HWND {
        self.0.hwnd
    }

    pub fn raw_uri(&self) -> LPCWSTR {
        self.0.uri
    }

    pub fn uri(&self) -> String {
        unsafe { utf::u16_ptr_to_string(self.0.uri) }
    }

    pub fn data(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.0.outData, self.0.outDataSize as usize) }
    }

    pub fn data_type(&self) -> SciterResourceType {
        SciterResourceType::from(self.0.dataType as i32)
    }

    pub fn request_id(&self) -> HREQUEST {
        self.0.requestId
    }
    pub fn principal(&self) -> HELEMENT {
        self.0.principal
    }
    pub fn initiator(&self) -> HELEMENT {
        self.0.initiator
    }
}

impl<'a> From<&'a SCN_LOAD_DATA> for LoadData<'a> {
    fn from(value: &'a SCN_LOAD_DATA) -> Self {
        Self(value)
    }
}

#[repr(transparent)]
pub struct DataLoaded<'a>(&'a SCN_DATA_LOADED);
impl<'a> DataLoaded<'a> {
    pub fn hwnd(&self) -> HWND {
        self.0.hwnd
    }

    pub fn raw_uri(&self) -> LPCWSTR {
        self.0.uri
    }

    pub fn uri(&self) -> String {
        unsafe { utf::u16_ptr_to_string(self.0.uri) }
    }

    pub fn data(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.0.data, self.0.dataSize as usize) }
    }

    pub fn data_type(&self) -> SciterResourceType {
        SciterResourceType::from(self.0.dataType as i32)
    }

    pub fn status(&self) -> u32 {
        // TODO: wrap status value:
        // status = 0 (dataSize == 0) - unknown error.
        // status = 100..505 - http response status, Note: 200 - OK!
        // status > 12000 - wininet error code, see ERROR_INTERNET_*** in wininet.h

        self.0.status
    }
}

impl<'a> From<&'a SCN_DATA_LOADED> for DataLoaded<'a> {
    fn from(value: &'a SCN_DATA_LOADED) -> Self {
        Self(value)
    }
}

pub struct AttachBehaviorRequest<'a>(&'a SCN_ATTACH_BEHAVIOR);

impl<'a> From<&'a SCN_ATTACH_BEHAVIOR> for AttachBehaviorRequest<'a> {
    fn from(value: &'a SCN_ATTACH_BEHAVIOR) -> Self {
        Self(value)
    }
}

impl<'a> AttachBehaviorRequest<'a> {
    pub fn hwnd(&self) -> HWND {
        self.0.hwnd
    }

    pub fn raw_behavior_name(&self) -> LPCSTR {
        self.0.behaviorName
    }

    pub fn behavior_name_cstr(&self) -> &'a CStr {
        unsafe { CStr::from_ptr(self.0.behaviorName as *const _) }
    }

    pub fn behavior_name(&self) -> Cow<'a, str> {
        self.behavior_name_cstr().to_string_lossy()
    }
}

pub struct AttachBehaviorResponse;
impl AttachBehaviorResponse {
    fn tag(&self) -> LPVOID {
        todo!()
    }
}

impl SciterResourceType {
    const UNKNOWN: SciterResourceType = SciterResourceType::RT_DATA_FORCE_DWORD;
}

impl From<i32> for SciterResourceType {
    fn from(value: i32) -> Self {
        match value {
            x if SciterResourceType::RT_DATA_HTML as i32 == x => SciterResourceType::RT_DATA_HTML,
            x if SciterResourceType::RT_DATA_IMAGE as i32 == x => SciterResourceType::RT_DATA_IMAGE,
            x if SciterResourceType::RT_DATA_STYLE as i32 == x => SciterResourceType::RT_DATA_STYLE,
            x if SciterResourceType::RT_DATA_CURSOR as i32 == x => {
                SciterResourceType::RT_DATA_CURSOR
            }
            x if SciterResourceType::RT_DATA_SCRIPT as i32 == x => {
                SciterResourceType::RT_DATA_SCRIPT
            }
            x if SciterResourceType::RT_DATA_RAW as i32 == x => SciterResourceType::RT_DATA_RAW,
            x if SciterResourceType::RT_DATA_FONT as i32 == x => SciterResourceType::RT_DATA_FONT,
            x if SciterResourceType::RT_DATA_SOUND as i32 == x => SciterResourceType::RT_DATA_SOUND,

            _ => SciterResourceType::UNKNOWN, //
        }
    }
}

pub(super) unsafe extern "C" fn host_thunk(
    pnm: LPSCITER_CALLBACK_NOTIFICATION,
    param: LPVOID,
) -> UINT {
    if !param.is_null() {
        let state_ptr = param as *mut WindowState;
        if let Some(host) = (*state_ptr).host() {
            let code = (*pnm).code;
            let hwnd = (*pnm).hwnd;
            match code {
                SC_LOAD_DATA => {
                    let (res, out_data) =
                        host.on_load_data(&LoadData::from(&*(pnm as LPSCN_LOAD_DATA)));
                    if let Some(out) = out_data.as_ref() {
                        let data = &mut *(pnm as LPSCN_LOAD_DATA);
                        data.outData = out.as_ptr();
                        data.outDataSize = out.len() as u32;
                    }

                    return res as UINT;
                }

                SC_DATA_LOADED => {
                    let data = DataLoaded::from(&*(pnm as LPSCN_DATA_LOADED));
                    host.on_data_loaded(&data);
                }

                SC_ATTACH_BEHAVIOR => {
                    let request = AttachBehaviorRequest::from(&*(pnm as LPSCN_ATTACH_BEHAVIOR));
                    if let Some(response) = host.on_attach_behavior(request) {
                        let data = &mut *(pnm as LPSCN_ATTACH_BEHAVIOR);
                        data.elementTag = response.tag();
                        data.elementProc = Some(element_proc_thunk);
                    }
                }

                SC_ENGINE_DESTROYED => {
                    host.on_engine_destroyed(hwnd);
                }

                SC_POSTED_NOTIFICATION => {
                    let notification = &mut *(pnm as LPSCN_POSTED_NOTIFICATION);
                    if let Some(res) = host.on_posted_notification(
                        notification.hwnd,
                        notification.wparam,
                        notification.lparam,
                    ) {
                        notification.lreturn = res;
                    }
                }

                SC_GRAPHICS_CRITICAL_FAILURE => {
                    host.on_graphics_critical_failure(hwnd);
                }

                _ => {
                    host.on_unknown(hwnd, code);
                }
            }
        }
    }

    0
}

unsafe extern "C" fn element_proc_thunk(
    tag: LPVOID,
    he: HELEMENT,
    event_group: UINT,
    params: LPVOID,
) -> SBOOL {
    let _ = tag;
    let _ = he;
    let _ = event_group;
    let _ = params;

    0
}

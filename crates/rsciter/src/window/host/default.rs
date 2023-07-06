use std::collections::HashMap;

use crate::{bindings::*, Archive, Result, Value};

use super::{AttachBehaviorRequest, AttachBehaviorResponse, HostNotifications};

pub enum ArchiveData {
    Static(&'static [u8]),
    Heap(Vec<u8>),
}

pub trait XFunction: 'static {
    fn call(&mut self, args: &[crate::Value]) -> Result<Option<Value>>;
}

impl<T> XFunction for T
where
    T: Fn(&[crate::Value]) -> Result<Option<Value>> + 'static,
{
    fn call(&mut self, args: &[crate::Value]) -> Result<Option<Value>> {
        (self)(args)
    }
}

pub trait XFunctionProvider: 'static {
    fn call(&mut self, name: &str, args: &[crate::Value]) -> Result<Option<Value>>;
}

pub struct DefaultHost {
    archive_uri: String,
    archive: Option<Archive>,
    functions: HashMap<String, Box<dyn XFunction>>,
}

impl Default for DefaultHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultHost {
    pub fn new() -> Self {
        Self::with_archive_uri("this://app/".to_string())
    }

    pub fn with_archive_uri(uri: String) -> Self {
        Self {
            archive_uri: uri,
            archive: None,
            functions: Default::default(),
        }
    }

    pub fn set_archive(&mut self, archive_data: ArchiveData) -> Result<()> {
        let archive = match archive_data {
            ArchiveData::Static(s) => Archive::open_static(s)?,
            ArchiveData::Heap(v) => Archive::open(v)?,
        };

        self.archive = Some(archive);
        Ok(())
    }

    pub(crate) fn set_functions(&mut self, functions: HashMap<String, Box<dyn XFunction>>) {
        self.functions = functions
    }
}

impl HostNotifications for DefaultHost {
    fn on_load_data(&mut self, data: &crate::LoadData) -> (crate::LoadDataResult, Option<&[u8]>) {
        if let Some(ref archive) = self.archive {
            let uri = data.uri();
            if uri.starts_with(&self.archive_uri) {
                let (_, rest) = uri.split_at(self.archive_uri.len());
                if let Ok(item) = archive.get(rest) {
                    return (crate::LoadDataResult::LOAD_OK, Some(item));
                }
            }
        }

        (crate::LoadDataResult::LOAD_OK, None)
    }

    fn on_data_loaded(&mut self, data: &crate::DataLoaded) {
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

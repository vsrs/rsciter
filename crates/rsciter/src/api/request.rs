use super::call_method;
use crate::{bindings, Error, Result};

#[derive(Debug, Clone, Copy)]
pub struct RequestApi<'api> {
    raw: &'api bindings::SciterRequestAPI,
}

impl<'api> From<&'api bindings::SciterRequestAPI> for RequestApi<'api> {
    fn from(value: &'api bindings::SciterRequestAPI) -> Self {
        Self { raw: value }
    }
}

impl RequestApi<'_> {
    pub fn r#use(&self, request: bindings::HREQUEST) -> Result<()> {
        call_method!(self, RequestUse(request))?.into()
    }
    pub fn unuse(&self, request: bindings::HREQUEST) -> Result<()> {
        call_method!(self, RequestUnUse(request))?.into()
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum RequestError {
    Panic = bindings::REQUEST_RESULT::REQUEST_PANIC as i32,
    BadParam = bindings::REQUEST_RESULT::REQUEST_BAD_PARAM as i32,
    Failure = bindings::REQUEST_RESULT::REQUEST_FAILURE as i32,
    NotSupported = bindings::REQUEST_RESULT::REQUEST_NOTSUPPORTED as i32,
}

impl From<bindings::REQUEST_RESULT> for Result<()> {
    fn from(value: bindings::REQUEST_RESULT) -> Self {
        match value {
            bindings::REQUEST_RESULT::REQUEST_OK => Ok(()),

            bindings::REQUEST_RESULT::REQUEST_PANIC => Err(Error::BadRequest(RequestError::Panic)),
            bindings::REQUEST_RESULT::REQUEST_BAD_PARAM => {
                Err(Error::BadRequest(RequestError::BadParam))
            }
            bindings::REQUEST_RESULT::REQUEST_FAILURE => {
                Err(Error::BadRequest(RequestError::Failure))
            }
            bindings::REQUEST_RESULT::REQUEST_NOTSUPPORTED => {
                Err(Error::BadRequest(RequestError::NotSupported))
            }
        }
    }
}

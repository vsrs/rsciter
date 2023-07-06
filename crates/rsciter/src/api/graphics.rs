use super::call_method;
use crate::{bindings, Error, Result};

#[derive(Debug, Clone, Copy)]
pub struct GraphicsApi<'api> {
    raw: &'api bindings::SciterGraphicsAPI,
}

impl<'api> From<&'api bindings::SciterGraphicsAPI> for GraphicsApi<'api> {
    fn from(value: &'api bindings::SciterGraphicsAPI) -> Self {
        Self { raw: value }
    }
}

impl GraphicsApi<'_> {
    pub fn image_add_ref(&self, himg: bindings::HIMG) -> Result<()> {
        let res = call_method!(self, imageAddRef(himg))?;
        res.into()
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GraphinError {
    Panic = bindings::GRAPHIN_RESULT::GRAPHIN_PANIC as i32,
    BadParam = bindings::GRAPHIN_RESULT::GRAPHIN_BAD_PARAM as i32,
    Failure = bindings::GRAPHIN_RESULT::GRAPHIN_FAILURE as i32,
    NotSupported = bindings::GRAPHIN_RESULT::GRAPHIN_NOTSUPPORTED as i32,
}

impl From<bindings::GRAPHIN_RESULT> for Result<()> {
    fn from(value: bindings::GRAPHIN_RESULT) -> Self {
        match value {
            bindings::GRAPHIN_RESULT::GRAPHIN_OK => Ok(()),
            bindings::GRAPHIN_RESULT::GRAPHIN_PANIC => {
                Err(Error::GraphinError(GraphinError::Panic))
            }
            bindings::GRAPHIN_RESULT::GRAPHIN_BAD_PARAM => {
                Err(Error::GraphinError(GraphinError::BadParam))
            }
            bindings::GRAPHIN_RESULT::GRAPHIN_FAILURE => {
                Err(Error::GraphinError(GraphinError::Failure))
            }
            bindings::GRAPHIN_RESULT::GRAPHIN_NOTSUPPORTED => {
                Err(Error::GraphinError(GraphinError::NotSupported))
            }
        }
    }
}

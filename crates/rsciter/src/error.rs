use std::num::TryFromIntError;

use crate::api::{GraphinError, RequestError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("'{dll}' loading error: {message}")]
    Library {
        dll: String,
        message: String,
        source: libloading::Error,
    },

    #[error("No '{name}' exported: {message}")]
    Symbol {
        name: String,
        message: String,
        source: libloading::Error,
    },

    #[error("'{0}' API method unavailable")]
    ApiMethod(&'static str),

    #[error("'{0}' API method call failed")]
    ApiMethodFailed(&'static str),

    #[error("Invalid atom value: '{0}'")]
    InvalidAtom(crate::bindings::som_atom_t),

    #[error("Request error: '{0:?}'")]
    BadRequest(RequestError),

    #[error("Graphin error: '{0:?}'")]
    GraphinError(GraphinError),

    #[error("Invalid archive data'")]
    InvalidArchive,

    #[error("Archive item {0} not found'")]
    ArchiveItemNotFound(String),

    #[error("Script evaluation failed")]
    EvalFailed,

    #[error("Value error: {0}")]
    ValueError(#[from] ValueError),

    #[error("Script error: {0}")]
    ScriptError(String),

    #[error("Scripting error, no such method: {0}")]
    ScriptingNoMethod(String),

    #[error("Scripting error, invalid arguments count: {0}")]
    ScriptingInvalidArgCount(String),

    #[error("Scripting error, invalid argument: {0}: {1}")]
    ScriptingInvalidArgument(&'static str, Box<Error>),

    #[error("unknown Sciter error")]
    Unknown,
}

#[cfg_attr(feature = "static", allow(dead_code))]
impl Error {
    pub(crate) fn library(name: &str, err: libloading::Error) -> Self {
        Self::Library {
            dll: name.to_string(),
            message: Self::get_message(&err),
            source: err,
        }
    }

    pub(crate) fn symbol(name: &str, err: libloading::Error) -> Self {
        Self::Symbol {
            name: name.to_string(),
            message: Self::get_message(&err),
            source: err,
        }
    }

    fn get_message(err: &libloading::Error) -> String {
        match std::error::Error::source(&err) {
            Some(src) => format!("{src}"),
            None => format!("{err}"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ValueError {
    #[error("Bad parameters")]
    BadParameters,

    #[error("Incompatible type")]
    IncompatibleType,

    #[error("from_string did not parse the rest: {0}")]
    FromStringNonParsed(u32),

    #[error("{0}")]
    TryFromIntError(#[from] TryFromIntError),

    #[error("Unknown: {0}")]
    Unknown(i32),
}

impl From<crate::bindings::VALUE_RESULT> for ValueError {
    fn from(value: crate::bindings::VALUE_RESULT) -> Self {
        match value {
            crate::bindings::VALUE_RESULT::HV_BAD_PARAMETER => ValueError::BadParameters,
            crate::bindings::VALUE_RESULT::HV_INCOMPATIBLE_TYPE => ValueError::IncompatibleType,
            unknown => ValueError::Unknown(unknown.0),
        }
    }
}

use core::str;
use std::{ffi::CStr, num::NonZero};

use crate::{api::sapi, bindings::*, Result};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Atom(NonZero<som_atom_t>);

impl Atom {
    pub fn new(name: impl AsRef<CStr>) -> Result<Self> {
        sapi()?
            .atom_value(name.as_ref())
            .map(|v| Self(unsafe { NonZero::new_unchecked(v) }))
    }

    pub fn name(&self) -> Result<String> {
        let mut target = String::new();
        let done =
            sapi()?.atom_name_cb(self.0.get(), Some(str_thunk), &mut target as *mut _ as _)?;
        if done {
            Ok(target)
        } else {
            Err(crate::Error::InvalidAtom(self.0.get()))
        }
    }
}

unsafe extern "C" fn str_thunk(data: LPCSTR, len: UINT, target_ptr: LPVOID) {
    let data = core::slice::from_raw_parts(data as _, len as _);
    let data = str::from_utf8_unchecked(data);
    let target = target_ptr as *mut String;
    *target = data.to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom() {
        let atom = Atom::new(c"name").unwrap();
        let name = atom.name().unwrap();
        assert_eq!(name, "name");
    }
}

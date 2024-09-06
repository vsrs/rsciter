use std::{
    ffi::CStr,
    marker::PhantomData,
    num::NonZero,
    os::raw::{c_char, c_long, c_void},
    ptr::NonNull,
    slice, str,
};

use crate::{api::sapi, bindings::*, Result, Value};

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

impl From<Atom> for som_atom_t {
    fn from(value: Atom) -> Self {
        value.0.get()
    }
}

unsafe extern "C" fn str_thunk(data: LPCSTR, len: UINT, target_ptr: LPVOID) {
    let data = slice::from_raw_parts(data as _, len as _);
    let data = str::from_utf8_unchecked(data);
    let target = target_ptr as *mut String;
    *target = data.to_string();
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct AssetObj(NonNull<som_asset_class_t>);

impl Clone for AssetObj {
    fn clone(&self) -> Self {
        let _res = self.add_ref();
        debug_assert_ne!(-1, _res, "No asset_add_ref!");

        Self(self.0)
    }
}

impl Drop for AssetObj {
    fn drop(&mut self) {
        let _res = self.release();
        debug_assert_ne!(-1, _res, "No asset_release!");
    }
}

impl AssetObj {
    pub(crate) fn new(class_data: som_asset_class_t) -> Self {
        let isa = Box::new(class_data);
        Self(unsafe { NonNull::new_unchecked(Box::leak(isa)) })
    }

    pub(crate) fn add_ref(&self) -> c_long {
        unsafe {
            let Some(f) = self.0.as_ref().asset_add_ref else {
                return -1;
            };

            f(core::mem::transmute_copy(self))
        }
    }

    pub(crate) fn release(&self) -> c_long {
        unsafe {
            let Some(f) = self.0.as_ref().asset_release else {
                return -1;
            };

            f(core::mem::transmute_copy(self))
        }
    }

    pub fn passport(&self) -> Option<&som_passport_t> {
        unsafe {
            self.0
                .as_ref()
                .asset_get_passport
                .map(|f| &*f(core::mem::transmute_copy(self)))
        }
    }

    // TODO: enumerate properties & methods, call methods
}

pub trait HasPassport {
    fn passport(&self) -> Result<&'static som_passport_t>;
}

pub trait ItemGetter : HasPassport {
    fn get_item(&self, key: Value) -> Result<Option<Value>>;
}

pub trait HasItemGetter {
    fn has_item_getter(&self) -> bool;
    fn do_get_item(&self, key: Value) -> Result<Option<Value>>;
}

impl<T> HasItemGetter for &&T {
    fn has_item_getter(&self) -> bool {
        false
    }
    fn do_get_item(&self, key: Value) -> Result<Option<Value>> {
        _ = key;
        Ok(None)
    }
}

impl<T: ItemGetter> HasItemGetter for &mut &&T {
    fn has_item_getter(&self) -> bool {
        true
    }
    fn do_get_item(&self, key: Value) -> Result<Option<Value>> {
        self.get_item(key)
    }
}

pub trait IAsset<T: HasPassport>: Sized {
    fn obj(&self) -> AssetObj;
    fn class() -> som_asset_class_t;
}

pub struct GlobalAsset<T: HasPassport> {
    obj: AssetObj,
    _t: PhantomData<T>,
}

impl<T: HasPassport> IAsset<T> for GlobalAsset<T> {
    fn obj(&self) -> AssetObj {
        self.obj.clone()
    }

    fn class() -> som_asset_class_t {
        // global assets are not ref-counted.
        unsafe extern "C" fn ref_count_stub(_thing: *mut som_asset_t) -> c_long {
            return 1;
        }

        unsafe extern "C" fn asset_get_interface(
            _thing: *mut som_asset_t,
            _name: *const c_char,
            _out: *mut *mut c_void,
        ) -> c_long {
            return 0;
        }

        unsafe extern "C" fn asset_get_passport<T: HasPassport>(
            thing: *mut som_asset_t,
        ) -> *mut som_passport_t {
            let this = thing as *mut GlobalInner<T>;
            let this = &*this;
            let Ok(passport) = this.data.passport() else {
                return std::ptr::null_mut();
            };
            passport as *const _ as *mut _
        }

        som_asset_class_t {
            asset_add_ref: Some(ref_count_stub),
            asset_release: Some(ref_count_stub),
            asset_get_interface: Some(asset_get_interface),
            asset_get_passport: Some(asset_get_passport::<T>),
        }
    }
}

#[repr(C)]
struct GlobalInner<T> {
    _obj: AssetObj,
    data: T,
}

impl<T: HasPassport> GlobalAsset<T> {
    pub fn new(data: T) -> Result<Self> {
        let obj = AssetObj::new(Self::class());
        let res = GlobalInner {
            _obj: obj.clone(),
            data,
        };

        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed);

        // SciterSetGlobalAsset overrides assets, so it might return false only if there is no asset_get_passport callback,
        // as we always provide one, it's safe to ignore the result
        let _res = sapi()?.set_global_asset(ptr as _)?;
        debug_assert!(_res);

        Ok(Self {
            obj,
            _t: PhantomData,
        })
    }
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

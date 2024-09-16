use std::{
    ffi::CStr,
    num::NonZero,
    ops::{Deref, DerefMut},
    os::raw::{c_char, c_long, c_void},
    ptr::NonNull,
    slice, str,
    sync::atomic::Ordering,
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
        Self(unsafe { NonNull::new_unchecked(Box::into_raw(isa)) })
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

pub type Passport = crate::bindings::som_passport_t;
pub trait HasPassport {
    fn passport(&self) -> Result<&'static Passport>;
}

pub trait ItemGetter: HasPassport {
    fn get_item(&self, key: &Value) -> Result<Option<Value>>;
}
pub trait HasItemGetter {
    fn has_item_getter(&self) -> bool;
    fn do_get_item(&self, key: &Value) -> Result<Option<Value>>;
}

impl<T> HasItemGetter for &&T {
    #[inline(always)]
    fn has_item_getter(&self) -> bool {
        false
    }

    fn do_get_item(&self, _key: &Value) -> Result<Option<Value>> {
        Ok(None)
    }
}

impl<T: ItemGetter> HasItemGetter for &mut &&T {
    #[inline(always)]
    fn has_item_getter(&self) -> bool {
        true
    }

    fn do_get_item(&self, key: &Value) -> Result<Option<Value>> {
        self.get_item(key)
    }
}

pub trait ItemSetter: HasPassport {
    fn set_item(&self, key: &Value, value: &Value) -> Result<()>;
}

pub trait HasItemSetter {
    fn has_item_setter(&self) -> bool;
    fn do_set_item(&self, key: &Value, value: &Value) -> Result<()>;
}

impl<T> HasItemSetter for &&T {
    #[inline(always)]
    fn has_item_setter(&self) -> bool {
        false
    }

    fn do_set_item(&self, _key: &Value, _value: &Value) -> Result<()> {
        Ok(())
    }
}

impl<T: ItemSetter> HasItemSetter for &mut &&T {
    #[inline(always)]
    fn has_item_setter(&self) -> bool {
        true
    }

    fn do_set_item(&self, key: &Value, value: &Value) -> Result<()> {
        self.set_item(key, value)
    }
}

#[macro_export]
macro_rules! impl_item_getter {
    ($type:ty) => {
        impl_item_getter!($type, item_getter)
    };
    ($type:ty, $name:ident) => {
        unsafe extern "C" fn $name(
            thing: *mut ::rsciter::bindings::som_asset_t,
            p_key: *const ::rsciter::bindings::SCITER_VALUE,
            p_value: *mut ::rsciter::bindings::SCITER_VALUE,
        ) -> ::rsciter::bindings::SBOOL {
            let key = p_key.as_value_ref();
            let asset_ref = ::rsciter::som::AssetRef::<$type>::new(thing);
            let Ok(Some(res)) = (&mut &asset_ref.data()).do_get_item(key) else {
                return 0;
            };

            *p_value = res.take();
            return 1;
        }
    };
}

#[macro_export]
macro_rules! impl_item_setter {
    ($type:ty) => {
        impl_item_setter!($type, item_setter)
    };
    ($type:ty, $name:ident) => {
        unsafe extern "C" fn $name(
            thing: *mut ::rsciter::bindings::som_asset_t,
            p_key: *const ::rsciter::bindings::SCITER_VALUE,
            p_value: *const ::rsciter::bindings::SCITER_VALUE,
        ) -> ::rsciter::bindings::SBOOL {
            let key = p_key.as_value_ref();
            let value = p_value.as_value_ref();
            let asset_ref = ::rsciter::som::AssetRef::<$type>::new(thing);
            let Ok(_) = (&mut &asset_ref.data()).do_set_item(key, value) else {
                return 0;
            };

            return 1;
        }
    };
}

pub use impl_item_getter;
pub use impl_item_setter;

pub type PropertyDef = crate::bindings::som_property_def_t;
pub type PropertyAccessorDef = crate::bindings::som_property_def_t__bindgen_ty_1;
pub type PropertyAccessors = crate::bindings::som_property_def_t__bindgen_ty_1__bindgen_ty_1;
unsafe impl Sync for PropertyDef {}
unsafe impl Send for PropertyDef {}

#[macro_export]
macro_rules! impl_prop {
    ($type:ident :: $name:ident) => {
        impl_prop!($type :: $name : true true)
    };
    ($type:ident :: $name:ident get) => {
        impl_prop!($type :: $name : true false)
    };
    ($type:ident :: $name:ident set) => {
        impl_prop!($type :: $name : false true)
    };
    ($type:ident :: $name:ident get set) => {
        impl_prop!($type :: $name : true true)
    };
    ($type:ident :: $name:ident set get) => {
        impl_prop!($type :: $name : true true)
    };

    ($type:ident :: $name:ident : $has_getter:literal $has_setter:literal) => {{
        use ::rsciter::*;

        unsafe extern "C" fn getter(
            thing: *mut bindings::som_asset_t,
            p_value: *mut bindings::SCITER_VALUE,
        ) -> bindings::SBOOL {
            let asset_ref = som::AssetRef::<$type>::new(thing);
            let Ok(value) = conv::ToValue::to_value(&asset_ref.$name) else {
                return 0;
            };

            *p_value = value.take();

            1
        }

        unsafe extern "C" fn setter(
            thing: *mut bindings::som_asset_t,
            p_value: *mut bindings::SCITER_VALUE,
        ) -> bindings::SBOOL {
            let mut asset_mut = som::AssetRefMut::<$type>::new(thing);
            let value = p_value.as_value_ref();
            let Ok(_) = ::rsciter::conv::FromValue::from_value(value)
                .map(|v| asset_mut.$name = v)
            else {
                return 0;
            };

            1
        }

        som::Atom::new(::rsciter_macro::cstr!($name)).map(|name| som::PropertyDef {
            type_: bindings::SOM_PROP_TYPE::SOM_PROP_ACCSESSOR.0 as _,
            name: name.into(),
            u: som::PropertyAccessorDef {
                accs: som::PropertyAccessors {
                    getter: if $has_getter { Some(getter) } else { None },
                    setter: if $has_setter { Some(setter) } else { None },
                },
            },
        })


    }};
}
pub use impl_prop;

/// There may be two property sources:
///
/// 1) Struct fields (handled via the [Fields] trait):
/// ```rust,ignore
/// #[rsciter::asset]
/// struct Asset {
///     name: String,
///     id: u32,
/// }
/// ```
///
/// 2) Virtual properties in `impl` blocks (handled via the [VirtualProperties] trait):
/// ```rust,ignore
/// #[rsciter::asset]
/// impl Asset {
///     #[get]
///     pub fn year(&self) -> String { ... }
///     #[set]
///     pub fn set_year(&self) -> String { ... }
/// }
/// ```
/// `property_name` and `set_property_name` patterns are handled automatically and bound to the same `property_name`.
/// Note: All public methods without `get` or `set` attributes are exported as functions.
///
/// Alternative syntax with explicit `year` name:
/// ```rust,ignore
/// #[rsciter::asset]
/// impl Asset {
///     #[get(year)]
///     fn any_get_year_name(&self) -> String { ... }
///
///     #[set(year)]
///     fn any_set_year_name(&self) -> String { ... }
/// }
/// ```
///
/// Note: The `get` and `set` attributes ignore visibility!
pub trait Fields: HasPassport {
    fn fields() -> &'static [Result<PropertyDef>];
}

/// See [Fields]. The traits are splitted only for codegen reasons.
pub trait VirtualProperties: HasPassport {
    fn properties() -> &'static [Result<PropertyDef>];
}

pub trait HasFields {
    fn enum_fields(&self) -> &'static [Result<PropertyDef>];
}

impl<T> HasFields for &&T {
    fn enum_fields(&self) -> &'static [Result<PropertyDef>] {
        &[]
    }
}

impl<T: Fields> HasFields for &mut &&T {
    fn enum_fields(&self) -> &'static [Result<PropertyDef>] {
        T::fields()
    }
}

pub trait HasVirtualProperties {
    fn enum_properties(&self) -> &'static [Result<PropertyDef>];
}

impl<T> HasVirtualProperties for &&T {
    fn enum_properties(&self) -> &'static [Result<PropertyDef>] {
        &[]
    }
}

impl<T: VirtualProperties> HasVirtualProperties for &mut &&T {
    fn enum_properties(&self) -> &'static [Result<PropertyDef>] {
        T::properties()
    }
}

pub type MethodDef = som_method_def_t;
unsafe impl Send for MethodDef {}
unsafe impl Sync for MethodDef {}
pub trait Methods: HasPassport {
    fn methods() -> &'static [Result<MethodDef>];
}

pub trait HasMethods {
    fn enum_methods(&self) -> &'static [Result<MethodDef>];
}

impl<T> HasMethods for &&T {
    fn enum_methods(&self) -> &'static [Result<MethodDef>] {
        &[]
    }
}

impl<T: Methods> HasMethods for &mut &&T {
    fn enum_methods(&self) -> &'static [Result<MethodDef>] {
        T::methods()
    }
}

pub trait IAsset<T: HasPassport>: Sized {
    fn class() -> som_asset_class_t;
}

pub struct GlobalAsset<T: HasPassport> {
    ptr: *mut AssetData<T>,
}

impl<T: HasPassport> IAsset<T> for GlobalAsset<T> {
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
            // TODO: query interface (any usage?)
            return 0;
        }

        unsafe extern "C" fn asset_get_passport<T: HasPassport>(
            thing: *mut som_asset_t,
        ) -> *mut som_passport_t {
            let asset_ref = AssetRef::<T>::new(thing);
            let Ok(passport) = asset_ref.passport() else {
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

impl<T: HasPassport> Drop for GlobalAsset<T> {
    fn drop(&mut self) {
        let ptr: *mut som_asset_t = self.ptr.cast();
        let _res = sapi().and_then(|api| api.release_global_asset(ptr));
        debug_assert!(_res.is_ok());
    }
}

impl<T: HasPassport> GlobalAsset<T> {
    pub fn new(data: T) -> Result<Self> {
        let obj = AssetObj::new(Self::class());
        let res = AssetData::new(obj, data);
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed);

        // SciterSetGlobalAsset overrides assets, so it might return false only if there is no asset_get_passport callback,
        // as we always provide one, it's safe to ignore the result
        let _res = sapi()?.set_global_asset(ptr as _)?;
        debug_assert!(_res);

        Ok(Self { ptr })
    }

    pub fn as_ref(&self) -> AssetRef<T> {
        unsafe { AssetRef::new(self.ptr.cast()) }
    }
}

#[macro_export]
macro_rules! impl_passport {
    ($self:ident, $type:ident) => {{
        static PASSPORT: std::sync::OnceLock<Result<bindings::som_passport_t>> =
            std::sync::OnceLock::new();

        let res = PASSPORT.get_or_init(|| {
            let mut passport =
                ::rsciter::bindings::som_passport_t::new(::rsciter_macro::cstr!($type))?;
            use ::rsciter::som::{
                self, HasFields, HasItemGetter, HasItemSetter, HasMethods, HasVirtualProperties,
            };

            let autoref_trick = &mut &$self;

            if autoref_trick.has_item_getter() {
                som::impl_item_getter!($type);
                passport.item_getter = Some(item_getter);
            }

            if autoref_trick.has_item_setter() {
                som::impl_item_setter!($type);
                passport.item_setter = Some(item_setter);
            }

            let mut properties = Vec::new();
            for f in autoref_trick.enum_fields() {
                match f {
                    Ok(v) => properties.push(v.clone()),
                    Err(e) => return Err(e.clone()),
                }
            }
            for p in autoref_trick.enum_properties() {
                match p {
                    Ok(v) => properties.push(v.clone()),
                    Err(e) => return Err(e.clone()),
                }
            }

            let mut methods = Vec::new();
            for m in autoref_trick.enum_methods() {
                match m {
                    Ok(v) => methods.push(v.clone()),
                    Err(e) => return Err(e.clone()),
                }
            }

            let boxed_props = properties.into_boxed_slice();
            passport.n_properties = boxed_props.len();
            if passport.n_properties > 0 {
                passport.properties = Box::into_raw(boxed_props) as *const _; // leak is acceptable here!
            }

            let boxed_methods = methods.into_boxed_slice();
            passport.n_methods = boxed_methods.len();
            if passport.n_methods > 0 {
                passport.methods = Box::into_raw(boxed_methods) as *const _; // leak is acceptable here!
            }

            Ok(passport)
        });

        match res {
            Ok(p) => Ok(p),
            Err(e) => Err(e.clone()),
        }
    }};
}
pub use impl_passport;

#[repr(C)]
struct AssetData<T> {
    obj: AssetObj,
    pub data: T,
}

impl<T> AssetData<T> {
    fn new(obj: AssetObj, data: T) -> Self {
        Self { obj, data }
    }
}

// TODO: refactor to support AsRef and Borrow
/// AssetRef does not use add_ref\release machinery,
/// instead it utilizes a lifetime and guranted to be a valid reference to an asset.
pub struct AssetRef<'a, T> {
    this: &'a AssetData<T>,
}

impl<'a, T> AssetRef<'a, T> {
    pub unsafe fn new(thing: *const som_asset_t) -> Self {
        let this = thing as *const AssetData<T>;
        let this = unsafe { &*this };
        Self { this }
    }

    pub fn data(&self) -> &T {
        &self.this.data
    }
}

impl<T> Deref for AssetRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data()
    }
}

pub struct AssetRefMut<'a, T> {
    this: &'a mut AssetData<T>,
}

impl<'a, T> AssetRefMut<'a, T> {
    pub unsafe fn new(thing: *mut som_asset_t) -> Self {
        let this = thing as *mut AssetData<T>;
        let this = unsafe { &mut *this };
        Self { this }
    }

    pub fn data(&self) -> &T {
        &self.this.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.this.data
    }
}

impl<T> Deref for AssetRefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data()
    }
}

impl<T> DerefMut for AssetRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

pub struct Asset<T: HasPassport> {
    boxed: Box<AssetDataWithCounter<T>>,
}

#[repr(C)]
struct AssetDataWithCounter<T> {
    data: AssetData<T>,
    counter: std::sync::atomic::AtomicI32,
}

impl<T> AssetDataWithCounter<T> {
    unsafe fn get_mut_ref<'c>(thing: *mut som_asset_t) -> &'c mut Self {
        let this = thing as *mut AssetDataWithCounter<T>;
        &mut *this
    }
}

impl<T: HasPassport> IAsset<T> for Asset<T> {
    fn class() -> som_asset_class_t {
        unsafe extern "C" fn asset_add_ref<TT>(thing: *mut som_asset_t) -> c_long {
            let this = AssetDataWithCounter::<TT>::get_mut_ref(thing);
            let refc = this.counter.fetch_add(1, Ordering::SeqCst) + 1;
            eprintln!("+ {thing:?} {refc}");
            return refc;
        }

        unsafe extern "C" fn asset_release<TT>(thing: *mut som_asset_t) -> c_long {
            let this = AssetDataWithCounter::<TT>::get_mut_ref(thing);
            let refc = this.counter.fetch_sub(1, Ordering::SeqCst) - 1;
            eprintln!("- {thing:?} {refc}");
            if refc == 0 {
                let _asset_to_drop = Box::from_raw(thing as *mut AssetDataWithCounter<TT>);
            }
            return refc;
        }

        unsafe extern "C" fn asset_get_interface(
            _thing: *mut som_asset_t,
            _name: *const c_char,
            _out: *mut *mut c_void,
        ) -> c_long {
            // TODO: query interface (any usage?)
            return 0;
        }

        unsafe extern "C" fn asset_get_passport<TT: HasPassport>(
            thing: *mut som_asset_t,
        ) -> *mut som_passport_t {
            let asset_ref = AssetRef::<TT>::new(thing);
            let Ok(passport) = asset_ref.passport() else {
                return std::ptr::null_mut();
            };
            passport as *const _ as *mut _
        }

        som_asset_class_t {
            asset_add_ref: Some(asset_add_ref::<T>),
            asset_release: Some(asset_release::<T>),
            asset_get_interface: Some(asset_get_interface),
            asset_get_passport: Some(asset_get_passport::<T>),
        }
    }
}

impl<T: HasPassport> Asset<T> {
    pub fn new(data: T) -> Self {
        let obj = AssetObj::new(Self::class());
        Self {
            boxed: Box::new(AssetDataWithCounter {
                data: AssetData::new(obj, data),
                counter: Default::default(),
            }),
        }
    }

    pub(crate) fn to_raw_ptr(self) -> *const som_asset_t {
        let ptr = Box::into_raw(self.boxed);
        ptr.cast()
    }

    pub fn as_ref(&self) -> AssetRef<T> {
        let ptr = &self.boxed.as_ref().data as *const AssetData<T>;
        unsafe { AssetRef::new(ptr.cast()) }
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

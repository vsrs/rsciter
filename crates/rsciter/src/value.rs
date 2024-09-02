use std::fmt::Debug;

use crate::{api::sapi, bindings::*, utf, Error, Result, ValueError};

#[repr(transparent)]
pub struct Value(pub(crate) VALUE);

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("Value");
        if let Ok(str) = self.to_string() {
            tuple.field(&str);
        } else {
            tuple.field(&"-");
        }

        tuple.finish()
    }
}

impl Eq for Value {}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // assume all values are different if sapi call failed
        sapi()
            .and_then(|api| api.value_compare(&self.0, &other.0))
            .unwrap_or(false)
    }
}

impl Value {
    pub fn bool(v: bool) -> Result<Self> {
        let mut this = Self::new();
        sapi()?.value_int_data_set(&mut this.0, if v { 1 } else { 0 }, VALUE_TYPE::T_BOOL, None)?;
        Ok(this)
    }

    pub fn int(v: i32) -> Result<Self> {
        let mut this = Self::new();
        sapi()?.value_int_data_set(&mut this.0, v, VALUE_TYPE::T_INT, None)?;
        Ok(this)
    }

    pub fn int64(v: i64) -> Result<Self> {
        let mut this = Self::new();
        sapi()?.value_int64_data_set(&mut this.0, v, VALUE_TYPE::T_BIG_INT, None)?;
        Ok(this)
    }

    pub fn float(v: f64) -> Result<Self> {
        let mut this = Self::new();
        sapi()?.value_float_data_set(&mut this.0, v, VALUE_TYPE::T_FLOAT, None)?;
        Ok(this)
    }

    pub fn string(str: impl AsRef<str>) -> Result<Self> {
        let mut this = Self::new();
        let str = utf::str_to_utf16_no_trailing_zero(str.as_ref());
        sapi()?.value_string_data_set(&mut this.0, &str, None)?;
        Ok(this)
    }

    pub fn bytes(bytes: impl AsRef<[u8]>) -> Result<Self> {
        let mut this = Self::new();
        sapi()?.value_binary_data_set(&mut this.0, bytes.as_ref(), VALUE_TYPE::T_BYTES, None)?;
        Ok(this)
    }

    pub fn array(data: &[Value]) -> Result<Self> {
        let mut this = Self::new();
        let api = sapi()?;
        api.value_int_data_set(&mut this.0, 0, VALUE_TYPE::T_ARRAY, None)?;
        for (idx, item) in data.iter().enumerate() {
            api.value_nth_element_value_set(&mut this.0, idx as i32, &item.0)?;
        }
        Ok(this)
    }

    pub fn array_from<'a, T>(data: impl Iterator<Item = &'a T>) -> Result<Self>
    where
        T: 'a,
        for<'i> &'i T: conv::ToValue,
    {
        let mut this = Self::new();
        let api = sapi()?;
        api.value_int_data_set(&mut this.0, 0, VALUE_TYPE::T_ARRAY, None)?;

        for (idx, item) in data.enumerate() {
            let item_val = conv::ToValue::to_value(item)?;
            api.value_nth_element_value_set(&mut this.0, idx as i32, &item_val.0)?;
        }

        Ok(this)
    }

    pub fn empty_array() -> Result<Self> {
        Self::array(&[])
    }

    pub fn map(data: &[(&Value, &Value)]) -> Result<Self> {
        let mut this = Self::new();
        let api = sapi()?;
        api.value_int_data_set(&mut this.0, 0, VALUE_TYPE::T_MAP, None)?;
        for (key, item) in data.iter() {
            api.value_set_value_to_key(&mut this.0, &key.0, &item.0)?;
        }
        Ok(this)
    }

    pub fn map_from<'a, K, V>(data: impl IntoIterator<Item = (&'a K, &'a V)>) -> Result<Self>
    where
        K: conv::ToValue + Copy + 'a,
        V: conv::ToValue + Copy + 'a,
    {
        let mut this = Self::new();
        let api = sapi()?;
        api.value_int_data_set(&mut this.0, 0, VALUE_TYPE::T_MAP, None)?;
        for (key, item) in data {
            let k = K::to_value(*key).unwrap();
            let v = V::to_value(*item).unwrap();

            api.value_set_value_to_key(&mut this.0, &k.0, &v.0)?;
        }
        Ok(this)
    }

    pub fn empty_map() -> Result<Self> {
        Self::map(&[])
    }

    pub fn secure_string(str: impl AsRef<str>) -> Result<Self> {
        let mut this = Self::new();
        let data = utf::str_to_utf16_no_trailing_zero(str.as_ref());
        sapi()?.value_string_data_set(
            &mut this.0,
            &data,
            Some(VALUE_UNIT_TYPE(VALUE_UNIT_TYPE_STRING::UT_STRING_SECURE.0)),
        )?;
        Ok(this)
    }

    pub fn error_string(str: impl AsRef<str>) -> Result<Self> {
        let mut this = Self::new();
        let data = utf::str_to_utf16_no_trailing_zero(str.as_ref());
        sapi()?.value_string_data_set(
            &mut this.0,
            &data,
            Some(VALUE_UNIT_TYPE(VALUE_UNIT_TYPE_STRING::UT_STRING_ERROR.0)),
        )?;
        Ok(this)
    }

    pub fn functor(f: impl NativeFunctor) -> Result<Self> {
        let mut this = Self::new();
        sapi()?.value_init(&mut this.0)?;
        let state = Box::new(FunctorState {
            functor: Box::new(f),
        });
        let tag = Box::into_raw(state);
        sapi()?.value_native_functor_set(
            &mut this.0,
            Some(functor_invoke_thunk),
            Some(functor_release_thunk),
            tag as LPVOID,
        )?;
        Ok(this)
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::new()
    }
}

// const block
impl Value {
    // ValueInit() call can be omitted if storage of the VALUE is initialized by all zeroes.
    // ValueInit(pval) is actually this memset(pval,0,sizeof(VALUE))
    // https://sciter.com/forums/reply/61441/

    const fn with_type_and_unit(t: UINT, u: UINT) -> Self {
        Self(VALUE { d: 0, t, u })
    }

    // aka void in script
    pub const NOTHING: Self = Self::with_type_and_unit(
        VALUE_TYPE::T_UNDEFINED.0 as UINT,
        VALUE_UNIT_UNDEFINED::UT_NOTHING.0 as UINT,
    );
    pub const UNDEFINED: Self = Self::with_type_and_unit(VALUE_TYPE::T_UNDEFINED.0 as UINT, 0);
    pub const NULL: Self = Self::with_type_and_unit(VALUE_TYPE::T_NULL.0 as UINT, 0);

    #[inline(always)]
    pub const fn new() -> Self {
        Self::UNDEFINED
    }

    pub const fn is_undefined(&self) -> bool {
        self.0.t == VALUE_TYPE::T_UNDEFINED.0 as UINT
    }

    pub const fn is_bool(&self) -> bool {
        self.0.t == VALUE_TYPE::T_BOOL.0 as UINT
    }

    pub const fn is_int(&self) -> bool {
        self.0.t == VALUE_TYPE::T_INT.0 as UINT
    }
    pub const fn is_float(&self) -> bool {
        self.0.t == VALUE_TYPE::T_FLOAT.0 as UINT
    }
    pub const fn is_string(&self) -> bool {
        self.0.t == VALUE_TYPE::T_STRING.0 as UINT
    }
    pub const fn is_symbol(&self) -> bool {
        self.is_string() && self.0.u == VALUE_UNIT_TYPE_STRING::UT_STRING_SYMBOL.0 as UINT
    }
    pub const fn is_error_string(&self) -> bool {
        self.is_string() && self.0.u == VALUE_UNIT_TYPE_STRING::UT_STRING_ERROR.0 as UINT
    }

    pub const fn is_date(&self) -> bool {
        self.0.t == VALUE_TYPE::T_DATE.0 as UINT
    }
    pub const fn is_big_int(&self) -> bool {
        self.0.t == VALUE_TYPE::T_BIG_INT.0 as UINT
    }
    pub const fn is_map(&self) -> bool {
        self.0.t == VALUE_TYPE::T_MAP.0 as UINT
    }
    pub const fn is_array(&self) -> bool {
        self.0.t == VALUE_TYPE::T_ARRAY.0 as UINT
    }
    pub const fn is_array_like(&self) -> bool {
        self.is_array()
            || (self.is_object() && self.0.u == VALUE_UNIT_TYPE_OBJECT::UT_OBJECT_ARRAY.0 as u32)
    }

    pub const fn is_function(&self) -> bool {
        self.0.t == VALUE_TYPE::T_FUNCTION.0 as UINT
    }
    pub const fn is_bytes(&self) -> bool {
        self.0.t == VALUE_TYPE::T_BYTES.0 as UINT
    }
    pub const fn is_object(&self) -> bool {
        self.0.t == VALUE_TYPE::T_OBJECT.0 as UINT
    }

    pub const fn is_asset(&self) -> bool {
        self.0.t == VALUE_TYPE::T_ASSET.0 as UINT
    }
    pub const fn is_color(&self) -> bool {
        self.0.t == VALUE_TYPE::T_COLOR.0 as UINT
    }
    pub const fn is_duration(&self) -> bool {
        self.0.t == VALUE_TYPE::T_DURATION.0 as UINT
    }
    pub const fn is_angle(&self) -> bool {
        self.0.t == VALUE_TYPE::T_ANGLE.0 as UINT
    }
    pub const fn is_null(&self) -> bool {
        self.0.t == VALUE_TYPE::T_NULL.0 as UINT && self.0.u == 0
    }
    pub const fn is_nothing(&self) -> bool {
        self.is_undefined() && self.0.u == VALUE_UNIT_UNDEFINED::UT_NOTHING.0 as UINT
    }
}

impl Value {
    pub fn make_copy(&self) -> Result<Self> {
        let mut new = Self::new();
        sapi()?.value_copy(&mut new.0, &self.0)?;
        Ok(new)
    }
    pub fn is_native_function(&self) -> Result<bool> {
        sapi()?.value_is_native_functor(&self.0)
    }

    pub fn get_bool(&self) -> Result<bool> {
        sapi()?.value_int_data(&self.0).map(|v| v != 0)
    }

    pub fn get_i32(&self) -> Result<i32> {
        sapi()?.value_int_data(&self.0)
    }

    pub fn get_u32(&self) -> Result<u32> {
        sapi()?.value_int_data(&self.0).map(|v| v as u32)
    }

    pub fn get_i16(&self) -> Result<i16> {
        sapi()?
            .value_int_data(&self.0)
            .and_then(|it| i16::try_from(it).map_err(|err| ValueError::from(err).into()))
    }

    pub fn get_u16(&self) -> Result<u16> {
        sapi()?
            .value_int_data(&self.0)
            .and_then(|it| u16::try_from(it).map_err(|err| ValueError::from(err).into()))
    }

    pub fn get_i64(&self) -> Result<i64> {
        sapi()?.value_int64_data(&self.0)
    }

    pub fn get_u64(&self) -> Result<u64> {
        sapi()?.value_int64_data(&self.0).map(|v| v as u64)
    }

    pub fn get_f64(&self) -> Result<f64> {
        sapi()?.value_float_data(&self.0)
    }

    pub fn get_string(&self) -> Result<String> {
        let api = sapi()?;
        let u16str = api.value_string_data(&self.0)?;
        Ok(String::from_utf16_lossy(u16str))
    }

    pub fn with_string_data(&self, f: impl Fn(&[u16])) -> Result<()> {
        let api = sapi()?;
        let u16str = api.value_string_data(&self.0)?;
        f(u16str);
        Ok(())
    }

    pub fn get_bytes(&self) -> Result<&[u8]> {
        sapi()?.value_binary_data(&self.0)
    }

    pub fn get_bytes_vec(&self) -> Result<Vec<u8>> {
        self.get_bytes().map(|v| v.to_vec())
    }

    pub fn get_color(&self) -> Result<i32> {
        // TODO: Color type
        if !self.is_color() {
            return Err(Error::from(ValueError::IncompatibleType));
        }

        sapi()?.value_int_data(&self.0)
    }

    pub fn get_angel(&self) -> Result<f64> {
        // TODO: Radians|Angel type
        if !self.is_angle() {
            return Err(Error::from(ValueError::IncompatibleType));
        }

        sapi()?.value_float_data(&self.0)
    }

    pub fn get_duration(&self) -> Result<f64> {
        // TODO: Duration type
        if !self.is_angle() {
            return Err(Error::from(ValueError::IncompatibleType));
        }

        sapi()?.value_float_data(&self.0)
    }

    pub fn get_date(&self) {
        todo!()
    }

    pub fn get_asset(&self) {
        todo!()
    }

    pub fn to_string(&self) -> Result<String> {
        self.to_string_as(ToStringKind::Simple)
    }

    pub fn to_string_as(&self, kind: ToStringKind) -> Result<String> {
        if self.is_string() && kind == ToStringKind::Simple {
            self.get_string()
        } else {
            let mut val = self.make_copy()?;
            sapi()?.value_to_string(&mut val.0, kind.into())?;
            val.get_string()
        }
    }

    // if it is an array or map returns number of elements there, otherwise - 0
    // if it is a function - returns number of arguments
    pub fn len(&self) -> Result<usize> {
        sapi()?.value_elements_count(&self.0)
    }

    pub fn is_empty(&self) -> Result<bool> {
        self.len().map(|v| v == 0)
    }

    // if it is an array - returns nth element
    // if it is a map - returns nth value of the map
    // if it is a function - returns nth argument
    // otherwise it returns undefined value
    pub fn get_item(&self, n: usize) -> Result<Value> {
        sapi()?.value_nth_element_value(&self.0, n as i32).map(Self)
    }

    pub fn get_item_by_key(&self, key: &Value) -> Result<Value> {
        sapi()?.value_get_value_of_key(&self.0, &key.0).map(Self)
    }

    pub fn get_item_by_name(&self, name: impl AsRef<str>) -> Result<Value> {
        let key = Self::string(name)?;
        self.get_item_by_key(&key)
    }

    pub fn enum_elements<'a>(
        &'a self,
        callback: impl Fn(&Value, &Value) -> bool + 'a,
    ) -> Result<()> {
        let state = EnumerateState {
            callback: Box::new(callback),
        };
        let ptr = &state as *const _;
        sapi()?.value_enum_elements(&self.0, enum_elements_thunk, ptr as LPVOID)
    }

    pub fn get_item_key(&self, n: usize) -> Result<Value> {
        sapi()?.value_nth_element_key(&self.0, n as i32).map(Self)
    }

    pub fn set_item(&mut self, n: usize, value: &Value) -> Result<()> {
        sapi()?.value_nth_element_value_set(&mut self.0, n as i32, &value.0)
    }

    pub fn set_item_by_key(&mut self, key: &Value, value: &Value) -> Result<()> {
        sapi()?.value_set_value_to_key(&mut self.0, &key.0, &value.0)
    }

    pub fn set_item_by_name(&mut self, name: impl AsRef<str>, value: &Value) -> Result<()> {
        let key = Self::string(name)?;
        self.set_item_by_key(&key, value)
    }

    pub fn with_object_data(&self, f: impl Fn(&[u8])) -> Result<()> {
        let api = sapi()?;
        let data = api.value_binary_data(&self.0)?;
        f(data);
        Ok(())
    }

    pub fn get_object_data(&self) -> Result<Vec<u8>> {
        sapi()?.value_binary_data(&self.0).map(|data| data.to_vec())
    }

    pub fn set_object_data(&mut self, data: &[u8]) -> Result<()> {
        sapi()?.value_binary_data_set(&mut self.0, data, VALUE_TYPE::T_OBJECT, None)
    }

    pub fn equals(&self, value: &Value) -> Result<bool> {
        if self.0.t == value.0.t && self.0.u == value.0.u && self.0.d == value.0.d {
            // strict comparison
            return Ok(true);
        }

        let compare_as = if self.0.t > value.0.t {
            self.0.t
        } else {
            value.0.t
        };
        match VALUE_TYPE(compare_as as i32) {
            // TODO: get_ with defaults to mimic value.hpp static bool equal
            VALUE_TYPE::T_BOOL => {
                let r1 = self.get_bool()?;
                let r2 = value.get_bool()?;

                Ok(r1 == r2)
            }

            VALUE_TYPE::T_INT => {
                let r1 = self.get_i32()?;
                let r2 = value.get_i32()?;

                Ok(r1 == r2)
            }

            VALUE_TYPE::T_FLOAT => {
                let r1 = self.get_f64()?;
                let r2 = value.get_f64()?;

                // most of the time it will fails
                // TODO: use float_cmp crate?
                Ok(r1 == r2)
            }

            _ => Ok(false),
        }
    }

    pub fn invoke(&self, this: Option<&mut Value>, args: &[Value]) -> Result<Option<Self>> {
        let mut this_stub = Self::new();
        let this = this.map_or(&mut this_stub.0, |v| &mut v.0);
        let args: Vec<VALUE> = args.iter().map(|it| it.0).collect();

        let mut ret_val = Self::new();
        sapi()?.value_invoke(&self.0, this, &args, &mut ret_val.0, 0 as _)?;

        if !ret_val.is_undefined() {
            Ok(Some(ret_val))
        } else {
            Ok(None)
        }
    }

    /// Consumes self
    pub fn take(self) -> VALUE {
        let res = self.0;
        std::mem::forget(self);
        res
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        if let Ok(api) = sapi() {
            let _ = api.value_clear(&mut self.0);
        }
    }
}

pub mod conv;

struct EnumerateState<'a> {
    #[allow(clippy::type_complexity)]
    callback: Box<dyn Fn(&Value, &Value) -> bool + 'a>,
}

unsafe extern "C" fn enum_elements_thunk(
    param: LPVOID,
    key: *const VALUE,
    value: *const VALUE,
) -> SBOOL {
    let state = &*(param as *const EnumerateState);
    let key = &*(key as *const Value); // Value is repr(transparent)
    let value = &*(value as *const Value);

    let res = (state.callback)(key, value);

    if res {
        // continue enumeration
        1
    } else {
        0
    }
}

pub trait NativeFunctor: 'static {
    fn invoke(&mut self, args: &[Value]) -> Option<Value>;
}

impl<T: Fn(&[Value]) -> Option<Value> + 'static> NativeFunctor for T {
    fn invoke(&mut self, args: &[Value]) -> Option<Value> {
        self(args)
    }
}

struct FunctorState {
    functor: Box<dyn NativeFunctor>,
}

unsafe extern "C" fn functor_invoke_thunk(
    tag: *mut ::std::os::raw::c_void,
    argc: UINT,
    argv: *const VALUE,
    retval: *mut VALUE,
) {
    let state = &mut *(tag as *mut FunctorState);
    let args = args_from_raw_parts(argv, argc);
    if let Some(res) = state.functor.invoke(args) {
        if !retval.is_null() {
            *retval = res.take();
        }
    }
}

unsafe extern "C" fn functor_release_thunk(tag: *mut ::std::os::raw::c_void) {
    let boxed = Box::from_raw(tag as *mut FunctorState);
    drop(boxed);
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
pub enum ToStringKind {
    Simple = VALUE_STRING_CVT_TYPE::CVT_SIMPLE.0,
    JsonLiteral = VALUE_STRING_CVT_TYPE::CVT_JSON_LITERAL.0,
    JsonMap = VALUE_STRING_CVT_TYPE::CVT_JSON_MAP.0,
    XJsonLiteral = VALUE_STRING_CVT_TYPE::CVT_XJSON_LITERAL.0,
}

impl From<ToStringKind> for VALUE_STRING_CVT_TYPE {
    fn from(value: ToStringKind) -> Self {
        match value {
            ToStringKind::Simple => VALUE_STRING_CVT_TYPE::CVT_SIMPLE,
            ToStringKind::JsonLiteral => VALUE_STRING_CVT_TYPE::CVT_JSON_LITERAL,
            ToStringKind::JsonMap => VALUE_STRING_CVT_TYPE::CVT_JSON_MAP,
            ToStringKind::XJsonLiteral => VALUE_STRING_CVT_TYPE::CVT_XJSON_LITERAL,
        }
    }
}

pub(crate) fn args_from_raw_parts<'a>(argv: *const VALUE, argc: u32) -> &'a [Value] {
    if argv.is_null() || argc == 0 {
        return &[];
    }

    let argv = argv as *const Value; // Value has $[repr(transparent)]
    let slice = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
    slice
}

pub(crate) fn args_as_raw_slice(args: &[Value]) -> &[VALUE] {
    if args.is_empty() {
        return &[];
    }

    let ptr = args.as_ptr() as *const VALUE; // Value has $[repr(transparent)]
    let slice = unsafe { std::slice::from_raw_parts(ptr, args.len()) };
    slice
}

#[cfg(test)]
pub mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::conv::*;
    use super::*;

    #[test]
    fn test_new() {
        let val = Value::new();

        assert!(val.is_undefined());
        assert!(!val.is_nothing());
        assert!(!val.is_null());
    }

    #[test]
    #[should_panic]
    fn test_undefined_incompatible_bool() {
        let val = Value::new();
        let _v = val.get_bool().unwrap();
    }

    #[test]
    #[should_panic = "called `Result::unwrap()` on an `Err` value: ValueError(IncompatibleType)"]
    fn test_undefined_incompatible_int() {
        let val = Value::new();
        val.get_i32().unwrap();
    }

    #[test]
    fn test_bool() {
        let val = Value::bool(true).unwrap();

        assert_eq!(val.get_bool().unwrap(), true);
        assert_eq!(val.get_i32().unwrap(), 1);
        assert_eq!(val.get_u32().unwrap(), 1);

        let val = Value::bool(false).unwrap();
        assert_eq!(val.get_bool().unwrap(), false);
        assert_eq!(val.get_i32().unwrap(), 0);
    }

    #[test]
    #[should_panic = "called `Result::unwrap()` on an `Err` value: ValueError(IncompatibleType)"]
    fn test_bool_incompatible_int64() {
        let val = Value::bool(true).unwrap();

        val.get_u64().unwrap();
    }

    #[test]
    fn test_array() {
        let v1 = Value::int(1).unwrap();
        let v2 = Value::int(2).unwrap();
        let v_true = Value::bool(true).unwrap();
        let v_str = Value::string("str").unwrap();
        let val = Value::array(&[v1, v2, v_true, v_str]).unwrap();

        assert!(val.is_array());
        assert_eq!(val.len().unwrap(), 4);
        assert_eq!(val.get_item(0).unwrap().get_i32().unwrap(), 1);
        assert_eq!(val.get_item(1).unwrap().get_i32().unwrap(), 2);
        assert_eq!(val.get_item(2).unwrap().get_bool().unwrap(), true);
        assert_eq!(val.get_item(3).unwrap().get_string().unwrap(), "str");
        assert_eq!(val.to_string().unwrap(), "[1,2,true,\"str\"]");

        let v = RefCell::new(Vec::new());

        val.enum_elements(|key, value| {
            let key = key.to_string().unwrap();
            assert_eq!(key, "");

            let value = value.to_string().unwrap();
            v.borrow_mut().push(value);
            true
        })
        .unwrap();

        assert_eq!(*v.borrow(), ["1", "2", "true", "str"]);
    }

    #[test]
    fn test_map() {
        let v1 = Value::int(1).unwrap();
        let v2 = Value::int(2).unwrap();
        let v_true = Value::bool(false).unwrap();
        let v_str = Value::string("str").unwrap();
        let val = Value::map(&[(&v1, &v_str), (&v2, &v_true)]).unwrap();

        assert!(val.is_map());
        assert_eq!(val.len().unwrap(), 2);

        let item0 = val.get_item(0).unwrap(); // "str"
        assert!(item0.equals(&v_str).unwrap());

        let item1 = val.get_item(1).unwrap(); // false
        assert!(item1.equals(&v_true).unwrap());

        let k = RefCell::new(Vec::new());
        let v = RefCell::new(Vec::new());

        val.enum_elements(|key, value| {
            let key = key.to_string().unwrap();
            k.borrow_mut().push(key);

            let value = value.to_string().unwrap();
            v.borrow_mut().push(value);
            true
        })
        .unwrap();

        assert_eq!(*k.borrow(), ["1", "2"]);
        assert_eq!(*v.borrow(), ["str", "false"]);
    }

    #[test]
    fn test_take() {
        let val = Value::string("asdf".to_string()).unwrap();
        let v = val.take();

        // v should be still valid;
        let restored = Value(v);
        assert_eq!(restored.get_string().unwrap(), "asdf");
    }

    #[test]
    fn test_functor() {
        struct F {
            invoked: bool,
            dropped: Rc<RefCell<bool>>,
        }

        impl NativeFunctor for F {
            fn invoke(&mut self, _args: &[Value]) -> Option<Value> {
                self.invoked = true;
                None
            }
        }

        impl Drop for F {
            fn drop(&mut self) {
                assert!(self.invoked);

                *self.dropped.borrow_mut() = true;
            }
        }

        let dropped = Rc::new(RefCell::new(false));

        {
            let val = Value::functor(F {
                invoked: false,
                dropped: dropped.clone(),
            })
            .unwrap();
            assert!(val.is_native_function().unwrap());
            let res = val.invoke(None, &[]).unwrap();
            assert!(res.is_none());
        }

        assert_eq!(*dropped.borrow(), true);
    }

    #[test]
    fn test_functor_arg() {
        let func = Value::functor(|args: &[Value]| {
            assert_eq!(args.len(), 1);

            Some(Value::int(14).unwrap())
        })
        .unwrap();

        let res = func.invoke(None, &[Value::string("str").unwrap()]).unwrap();
        let res = res.unwrap();
        assert_eq!(res.get_i32().unwrap(), 14);
    }

    #[test]
    fn test_functor_args() {
        let func = Value::functor(|args: &[Value]| {
            assert_eq!(args.len(), 3);

            let mut arr = Value::empty_array().unwrap();

            for (idx, arg) in args.into_iter().enumerate() {
                arr.set_item(idx, arg).unwrap();
            }

            Some(arr)
        })
        .unwrap();

        let res = func
            .invoke(
                None,
                &[
                    Value::string("str").unwrap(),
                    Value::int(44).unwrap(),
                    Value::bool(false).unwrap(),
                ],
            )
            .unwrap();
        let res = res.unwrap();
        assert!(res.is_array());
        assert_eq!(res.to_string().unwrap(), "[\"str\",44,false]");
    }

    #[test]
    fn test_u64_from_value() {
        let val = Value::int(32).unwrap();
        let v: u64 = FromValue::from_value(&val).unwrap();

        assert_eq!(v, 32);
    }

    #[test]
    fn test_u64_to_value() {
        let val = ToValue::to_value(64).unwrap();
        let x: i32 = FromValue::from_value(&val).unwrap();

        assert_eq!(x, 64);
    }

    // TODO: more tests

    // TryFrom tests
    #[test]
    fn test_from_bool() {
        let val = Value::try_from(true).unwrap();
        assert_eq!(val, Value::bool(true).unwrap());
    }

    #[test]
    fn test_from_int() {
        let val = Value::try_from(12).unwrap();
        assert_eq!(val, Value::int(12).unwrap());
    }

    #[test]
    fn test_from_str() {
        let val = Value::try_from("sss").unwrap();
        assert_eq!(val, Value::string("sss").unwrap());
    }

    #[test]
    fn test_from_string() {
        let val = Value::try_from("SSS".to_string()).unwrap();
        assert_eq!(val, Value::string("SSS").unwrap());
    }

    #[test]
    fn test_from_int_array() {
        let val = Value::try_from([1, 2, 3, 4]).unwrap();
        let val_ref = Value::try_from(&[1, 2, 3, 4]).unwrap();

        assert!(val.is_array());
        assert_eq!(val.len().unwrap(), 4);

        assert_eq!(val, val_ref);
    }

    #[test]
    fn test_from_str_array_vec() {
        let val = Value::try_from(["1", "2", "3", "4"]).unwrap();
        let val_vec = Value::try_from(vec!["1", "2", "3", "4"]).unwrap();

        let val_ref = Value::try_from(&["1", "2", "3", "4"]).unwrap();
        let val_vec_ref = Value::try_from(&vec!["1", "2", "3", "4"]).unwrap();

        assert!(val.is_array());
        assert_eq!(val.len().unwrap(), 4);

        assert_eq!(val, val_vec);
        assert_eq!(val, val_ref);
        assert_eq!(val, val_vec_ref);
    }
}

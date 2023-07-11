use super::Value;
use crate::{Error, Result};

pub trait FromValue<'a>: Sized {
    fn from_value(value: &'a Value) -> Result<Self>;
}

pub trait ToValue: Sized {
    fn to_value(val: Self) -> Result<Value>;
}

macro_rules! impl_from {
    ($type:ty, $from:ident) => {
        impl FromValue<'_> for $type {
            fn from_value(value: &Value) -> Result<Self> {
                value.$from()
            }
        }

        impl TryFrom<Value> for $type {
            type Error = Error;

            fn try_from(val: Value) -> Result<Self> {
                FromValue::from_value(&val)
            }
        }

        impl TryFrom<&Value> for $type {
            type Error = Error;

            fn try_from(val: &Value) -> Result<Self> {
                FromValue::from_value(val)
            }
        }
    };
}

macro_rules! impl_primitive {
    ($type:ty, $from:ident, $val: ident, $to:expr) => {
        impl_from!($type, $from);

        impl TryFrom<$type> for Value {
            type Error = Error;

            fn try_from($val: $type) -> Result<Self> {
                $to
            }
        }

        impl ToValue for $type {
            fn to_value(val: Self) -> Result<Value> {
                Value::try_from(val)
            }
        }
    };
}

// https://doc.rust-lang.org/std/macro.concat_idents.html is unstable
impl_primitive!(bool, get_bool, val, Value::bool(val));
impl_primitive!(i16, get_i16, val, Value::int(val.into()));
impl_primitive!(u16, get_u16, val, Value::int(val.into()));
impl_primitive!(i32, get_i32, val, Value::int(val));
impl_primitive!(u32, get_u32, val, Value::int(val as i32));
impl_primitive!(i64, get_i64, val, Value::int64(val));
impl_primitive!(u64, get_u64, val, Value::int64(val as i64));

impl_from!(String, get_string);

impl TryFrom<&str> for Value {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        Value::string(val)
    }
}

impl ToValue for &str {
    fn to_value(val: Self) -> Result<Value> {
        Value::try_from(val)
    }
}

impl TryFrom<String> for Value {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        Value::string(&val)
    }
}

impl ToValue for String {
    fn to_value(val: Self) -> Result<Value> {
        Value::try_from(val)
    }
}

impl FromValue<'_> for Value {
    fn from_value(value: &Value) -> Result<Self> {
        value.make_copy()
    }
}

impl<'v> FromValue<'v> for &'v Value {
    fn from_value(value: &'v Value) -> Result<&'v Value> {
        Ok(value)
    }
}

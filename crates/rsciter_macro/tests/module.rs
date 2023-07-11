use rsciter::{Error, Value, XFunctionProvider};

#[rsciter::xmod]
mod NativeModule {
    use rsciter::{Value, Result};

    pub fn no_args() {}

    pub fn second(x: u64, x_ref: &u64) {
        dbg!(x, x_ref);
    }

    pub fn value(arg1: Value, arg_ref: &Value) {
        dbg!(arg1, arg_ref);
    }

    pub fn string(s: String, s_ref: &String, str_ref: &str) {
        dbg!(s, s_ref, str_ref);
    }

    #[allow(unused_mut)]
    pub fn mutable_arg(mut arg: u64) {
        dbg!(arg);
    }

    fn private() {}

    pub fn ret_i32() -> i32 {
        42
    }

    pub fn ret_value(arg: Value) -> Value {
        arg
    }

    pub fn ret_res_value() -> Result<Value> {
        Value::int(11)
    }
}

#[test]
fn test_private_is_unavailable() {
    let mut xfn_provider = NativeModule;
    let err = xfn_provider.call("private", &[]).unwrap_err();
    assert!(matches!(err, Error::ScriptingNoMethod(s) if s == "private"));
}

#[test]
fn test_no_args() {
    let mut xfn_provider = NativeModule;
    xfn_provider.call("no_args", &[]).unwrap();
}

#[test]
fn test_second_fail() {
    let mut xfn_provider = NativeModule;
    let err = xfn_provider.call("second", &[]).unwrap_err();
    assert!(matches!(err, Error::ScriptingInvalidArgCount(s) if s == "second"));
}

#[test]
fn test_second() {
    rsciter::update_path();

    let x = Value::int(12).unwrap();
    let x_ref = Value::int(33).unwrap();

    let mut xfn_provider = NativeModule;
    xfn_provider.call("second", &[x, x_ref]).unwrap();
}

#[test]
fn test_value() {
    rsciter::update_path();

    let v = Value::int(31).unwrap();
    let v_ref = Value::string("str").unwrap();

    let mut xfn_provider = NativeModule;
    xfn_provider.call("value", &[v, v_ref]).unwrap();
}

#[test]
fn test_string() {
    rsciter::update_path();

    let s = Value::string("String").unwrap();
    let s_ref = Value::string("&String").unwrap();
    let str_ref = Value::string("&str").unwrap();

    let mut xfn_provider = NativeModule;
    xfn_provider.call("string", &[s, s_ref, str_ref]).unwrap();
}

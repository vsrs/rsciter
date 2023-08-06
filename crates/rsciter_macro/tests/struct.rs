use rsciter::{Error, Result, Value, XFunctionProvider};

struct S;

#[rsciter::xmod]
impl S {
    fn private(&self) {}

    pub fn no_args(&self) {}

    pub fn second(&self, x: u64, x_ref: &u64) {
        dbg!(x, x_ref);
    }

    pub fn value(&mut self, arg1: Value, arg_ref: &Value) {
        dbg!(arg1, arg_ref);
    }

    pub fn string(&self, s: String, s_ref: &String, str_ref: &str) {
        dbg!(s, s_ref, str_ref);
    }

    #[allow(unused_mut)]
    pub fn mutable_arg(&mut self, mut arg: u64) {
        dbg!(arg);
    }

    pub fn ret_i32(&self) -> i32 {
        42
    }

    pub fn ret_value(&self, arg: Value) -> Value {
        arg
    }

    pub fn ret_res_value(&mut self) -> Result<Value> {
        Value::int(11)
    }
}

#[test]
fn test_private_is_unavailable() {
    let mut xfn_provider = S;
    let err = xfn_provider.call("private", &[]).unwrap_err();
    assert!(matches!(err, Error::ScriptingNoMethod(s) if s == "private"));
}

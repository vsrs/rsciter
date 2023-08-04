use rsciter::{Error, Value, XFunctionProvider};

struct S;

#[rsciter::xmod]
impl S {
    fn private(&self) {

    }
}


#[test]
#[ignore]
fn test_private_is_unavailable() {
    let mut xfn_provider = S;
    // let err = xfn_provider.call("private", &[]).unwrap_err();
    // assert!(matches!(err, Error::ScriptingNoMethod(s) if s == "private"));
}

use trybuild::TestCases;

#[test]
fn compile_fail() {
    let t = TestCases::new();
    t.compile_fail("tests/compile_fail/*_test.rs");
}

#[test]
fn compile_pass() {
    let t = TestCases::new();
    t.pass("tests/compile_pass/*_test.rs");
}

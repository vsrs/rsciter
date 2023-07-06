use rsciter::*;

fn main() {
    #[cfg(any(test, debug_assertions))]
    rsciter::update_path();

    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

// #[sciter::xmod(NativeModule)]
// mod native {
//     use rsciter::{Result,Value};

//     #[skip]
//     pub fn sum(a: u64, b: u64) -> u64 {
//         todo!()
//     }

//     pub fn stringize(args: &[Value]) -> Result<Option<Value>> {
//         todo!()
//     }

//     #[as(countZeroes)]
//     pub fn count_zeroes(args: &[Value]) -> Result<Option<Value>> {
//         todo!()
//     }
// }

fn try_main() -> Result<i32> {
    app::init()?;

    let _window = Window::builder()
        .with_function("printArgs", print_args)
        .with_function("return13", |_args: &[Value]| Value::int(13).map(Some))
        //        .with_module(NativeModule)
        .with_html(HTML)
        .build_main()?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

fn print_args(args: &[Value]) -> Result<Option<Value>> {
    for arg in args {
        println!("{}", arg.to_string_as(ToStringKind::JsonMap).unwrap());
        println!("{}", arg.to_string_as(ToStringKind::JsonLiteral).unwrap());
    }

    Ok(None)
}

const HTML: &[u8] = br#"
<html>
<head>
<script>
    Window.this.state = Window.WINDOW_SHOWN;

    printArgs(false);
    printArgs("asdf");
    printArgs({test: 12, arg: 'str', obj: { val: false }});
    
</script>
</head>
<body></body>
</html>
"#;

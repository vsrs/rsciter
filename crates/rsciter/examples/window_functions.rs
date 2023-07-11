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

#[rsciter::xmod]
mod NativeModule {
    pub fn sum(a: u64, b: u64) -> u64 {
        a + b
    }

    pub fn u64_to_str(a: u64) -> String {
        format!("{a}")
    }

    pub fn i64_to_str(a: i64) -> String {
        format!("{a}")
    }
}

fn try_main() -> Result<i32> {
    app::init()?;

    let _window = Window::builder()
        .with_function("printArgs", print_args)
        .with_function("return13", |_args: &[Value]| Value::int(13).map(Some))
        .with_module(NativeModule)
        .with_html(HTML)
        .build_main()?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

fn print_args(args: &[Value]) -> Result<Option<Value>> {
    for arg in args {
        println!("{}", arg.to_string_as(ToStringKind::JsonLiteral).unwrap());
    }

    Ok(None)
}

const HTML: &[u8] = br#"
<html>
<head>
<script>
    Window.this.state = Window.WINDOW_SHOWN;

    const sum = Window.this.xcall("sum", 12, 12);
    const u64 = Window.this.xcall("u64_to_str", 123456789);
    const i64 = Window.this.xcall("i64_to_str", -123456789);
    const closureValue = Window.this.xcall("return13");

    Window.this.xcall("printArgs", false, sum, u64, i64, closureValue);
   
</script>
</head>
<body></body>
</html>
"#;

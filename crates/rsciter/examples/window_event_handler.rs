use rsciter::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const HTML: &[u8] = br#"
<html>
<head>
<script>
    Window.this.state = Window.WINDOW_SHOWN;

    const printArgs = function(...args) {
        Window.this.xcall('printArgs', ...args);
    }

    printArgs(false);
    printArgs("asdf");
    printArgs({test: 12, arg: 'str', obj: { val: false }});
    
</script>
</head>
<body></body>
</html>
"#;

fn try_main() -> Result<i32> {
    app::init()?;

    let _window = Window::builder()
        .with_event_handler(Handler)
        .with_html(HTML)
        .build_main()?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

struct Handler;

fn print_args(args: &[Value]) -> Result<Option<Value>> {
    for arg in args {
        println!("{}", arg.to_string_as(ToStringKind::JsonMap).unwrap());
        println!("{}", arg.to_string_as(ToStringKind::JsonLiteral).unwrap());
    }

    Ok(None)
}

impl EventHandler for Handler {
    fn on_scripting_method_call(
        &mut self,
        he: bindings::HELEMENT,
        name: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let _ = he;

        if name == "printArgs" {
            return print_args(args);
        }

        Err(Error::ScriptingNoMethod(name.to_string()))
    }
}

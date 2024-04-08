use rsciter::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

struct StatefullApi {
    state: u64,
}

#[rsciter::xmod]
impl StatefullApi {
    pub fn sum(&self, a: u64, b: u64) -> u64 {
        a + b + self.state
    }

    pub fn update(&mut self, a: u64) {
        self.state = a;
    }

    pub fn state(&self) -> u64 {
        self.state
    }
}

fn try_main() -> Result<i32> {
    let _dbg = rsciter::setup_debug_output(|subsystem, severity, message| {
        println!("{subsystem:?}, {severity:?}: {message}");
    });

    app::init()?;

    let _window = Window::builder()
        .with_module(StatefullApi { state: 14 })
        .with_html(HTML)
        .build_main()?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

const HTML: &'static [u8] = br#"
<html>
<head>
<script>
    Window.this.state = Window.WINDOW_SHOWN;

    const sum = Window.this.xcall("sum", 12, 12);
    console.log("sum:", sum);

    console.log("state:", Window.this.xcall("state"));

    Window.this.xcall("update", 12342)
    console.log("state:", Window.this.xcall("state"));

    console.log("new sum:", Window.this.xcall("sum", 12, 12));

</script>
</head>
<body></body>
</html>
"#;

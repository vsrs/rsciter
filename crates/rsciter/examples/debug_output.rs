use rsciter::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const HTML: &[u8] = br##"<html>
<head>
<script>
    console.log("log message");
</script>
</head>
<body>See debug console</body>
</html>"##;

fn try_main() -> Result<i32> {
    app::init()?;

    let _v = setup_debug_output(|sub, sev, text| {
        eprintln!("Sub: {:?}, Level: {:?}, {text}", sub, sev);
    })?;

    let window = Window::builder()
        .with_html(HTML)
        .build_main()?;

    window.show(Visibility::Normal)?;

    app::run()
}
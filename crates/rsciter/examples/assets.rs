use rsciter::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const HTML: &'static [u8] = br#"<html>
<head>
  <script>
    Db.open("test.db", 0);
    Db.update("asdf");
  </script>
</head>

<body>
</body>

</html>"#;

#[rsciter::asset]
mod Db {
    pub fn open(path: &str, flags: u64) {
        println!("Opening: {path} with flags: {flags}");
    }

    pub fn update(path: &str) {
        println!("Updating: {path}");
    }
}

fn try_main() -> Result<i32> {
    app::init()?;

    let _v = setup_debug_output(|sub, sev, text| {
        eprintln!("Sub: {:?}, Level: {:?}, {text}", sub, sev);
    })?;

    // let _ = will drop the Db immediately!
    let _guard = som::GlobalAsset::new(Db)?;

    let window = Window::builder().with_html(HTML).build_main()?;
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    Ok(exit_code)
}

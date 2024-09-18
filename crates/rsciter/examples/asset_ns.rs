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
    const obj = Db.open("test.db", 4);
    console.log(`open result: "${obj}, ${obj.path}, ${obj.flags}"`);

    const updateRes = obj.update("value");
    console.log(updateRes, updateRes.message);

    console.log(`Update result: "${updateRes.message()}"`);
  </script>
</head>

<body>
</body>

</html>"#;

// a single macro to export entire backend
#[rsciter::asset_ns]
mod Db {
    pub fn open(path: &str, flags: u64) -> Object {
        Object {
            path: path.into(),
            flags,
        }
    }

    pub struct Object {
        path: String,
        flags: u64,
    }

    impl Object {
        pub fn update(&self, value: &str) -> UpdateRes {
            UpdateRes(format!(
                "Updating: {value} for {} with {}",
                self.path, self.flags
            ))
        }
    }

    // UpdateRes might be private, as it used only inside the backend
    struct UpdateRes(String);
    impl UpdateRes {
        pub fn message(&self) -> &str {
            &self.0
        }
    }
}

fn try_main() -> rsciter::Result<i32> {
    use rsciter::*;

    app::init()?;

    let _v = setup_debug_output(|sub, sev, text| {
        eprintln!("Sub: {:?}, Level: {:?}, {text}", sub, sev);
    })?;

    // let _ = will drop the Db immediately!
    let _guard = Db::new()?;

    let window = Window::builder().with_html(HTML).build_main()?;
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    Ok(exit_code)
}

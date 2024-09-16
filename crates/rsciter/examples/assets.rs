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

#[rsciter::asset]
struct Object {
    path: String,
    flags: u64,
}

#[rsciter::asset]
impl Object {
    pub fn update(&self, value: &str) -> UpdateRes {
        UpdateRes(format!(
            "Updating: {value} for {} with {}",
            self.path, self.flags
        ))
    }
}

struct UpdateRes(String);

// If a struct  itself does not have #[rsciter::asset] attribute,
// it's enough to specify #[rsciter::asset(HasPassport)] for impl block
#[rsciter::asset(HasPassport)]
impl UpdateRes {
    pub fn message(&self) -> &str {
        &self.0
    }
}

#[rsciter::asset_ns]
mod Db {
    use super::*;

    pub fn open(path: &str, flags: u64) -> Object {
        Object {
            path: path.into(),
            flags,
        }
    }
}

fn try_main() -> Result<i32> {
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

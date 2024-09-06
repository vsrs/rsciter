use rsciter::*;
use std::sync::OnceLock;

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
    console.log(Person);
    Person.test = 4;
  </script>
</head>

<body>
</body>

</html>"#;

#[derive(Default)]
pub struct Person {
    age: i32,
    name: String,
}

impl som::ItemGetter for Person {
    fn get_item(&self, key: &Value) -> Result<Option<Value>> {
        dbg!(key);

        Ok(None)
    }
}

impl som::ItemSetter for Person {
    fn set_item(&self, key: &Value, value: &Value) -> Result<()> {
        dbg!(key, value);

        Ok(())
    }
}

impl som::HasPassport for Person {
    fn passport(&self) -> Result<&'static bindings::som_passport_t> {
        static PASSPORT: OnceLock<Result<bindings::som_passport_t>> = OnceLock::new();

        let res = PASSPORT.get_or_init(|| {
            let mut passport = bindings::som_passport_t::new(c"Person")?;
            use som::HasItemGetter;
            use som::HasItemSetter;

            if (&mut &self).has_item_getter() {
                som::impl_item_getter!(Person);
                passport.item_getter = Some(item_getter);
            }

            if (&mut &self).has_item_setter() {
                som::impl_item_setter!(Person);
                passport.item_setter = Some(item_setter);
            }

            Ok(passport)
        });

        match res {
            Ok(p) => Ok(p),
            Err(e) => Err(e.clone()),
        }
    }
}

fn try_main() -> Result<i32> {
    app::init()?;

    let _v = setup_debug_output(|sub, sev, text| {
        eprintln!("Sub: {:?}, Level: {:?}, {text}", sub, sev);
    })?;

    let _person_glob = som::GlobalAsset::new(Person {
        age: 42,
        name: "Arthur".into(),
    })?;

    let window = Window::builder().with_html(HTML).build_main()?;
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

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
    if (typeof Person === 'undefined') {
      console.log("No Person");
    } else {
      console.log(Person);
      console.log("age: ", Person.age);
      console.log("name: ", Person.name);

      Person.test = 4;
      console.log(Person.test);
    }
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
        println!("Get item: {key:?}");

        Ok(None)
    }
}

impl som::ItemSetter for Person {
    fn set_item(&self, key: &Value, value: &Value) -> Result<()> {
        println!("Set item: {key:?} to `{value:?}`");
        Ok(())
    }
}

impl som::Fields for Person {
    fn fields() -> &'static [som::PropertyDef] {
        static FIELDS: std::sync::OnceLock<[som::PropertyDef; 2]> = std::sync::OnceLock::new();

        FIELDS.get_or_init(|| [
            som::impl_ro_prop!(Person::age),
            som::impl_ro_prop!(Person::name)
        ])
    }
}

impl som::HasPassport for Person {
    fn passport(&self) -> Result<&'static som::Passport> {
        static PASSPORT: std::sync::OnceLock<Result<bindings::som_passport_t>> =
            std::sync::OnceLock::new();

        let res = PASSPORT.get_or_init(|| {
            let mut passport = bindings::som_passport_t::new(c"Person")?;
            use som::{Fields, HasItemGetter, HasItemSetter};

            if (&mut &self).has_item_getter() {
                som::impl_item_getter!(Person);
                passport.item_getter = Some(item_getter);
            }

            if (&mut &self).has_item_setter() {
                som::impl_item_setter!(Person);
                passport.item_setter = Some(item_setter);
            }

            let fields = <Person as Fields>::fields();

            let mut properties = Vec::with_capacity(fields.len());
            for f in fields {
                properties.push(f.clone());
            }
            let boxed = properties.into_boxed_slice();
            passport.n_properties = boxed.len();
            passport.properties = Box::into_raw(boxed) as *const _;

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

    let person_asset = som::GlobalAsset::new(Person {
        age: 42,
        name: "Arthur".into(),
    })?;

    let window = Window::builder().with_html(HTML).build_main()?;
    window.show(Visibility::Normal)?;

    let _exit_code = app::run()?;

    // try to comment and see output
    drop(person_asset);

    eprintln!("Start second session (with dropped asset)");

    let window = Window::builder().with_html(HTML).build_main()?;
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

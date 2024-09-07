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

      Person.age = 13;
      console.log("new age: ", Person.age);

      const formatted = Person.format();
      console.log(`Formatted: "${formatted}"`);
    }
  </script>
</head>

<body>
</body>

</html>"#;

#[derive(Default, Debug)]
pub struct Person {
    age: i32,
    name: String,
}

impl Person {
    pub fn format(&self) -> String {
        format!("{} of {}", &self.name, self.age)
    }
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
    fn fields() -> &'static [Result<som::PropertyDef>] {
        static FIELDS: std::sync::OnceLock<[Result<som::PropertyDef>; 2]> =
            std::sync::OnceLock::new();

        FIELDS.get_or_init(|| [som::impl_prop!(Person::age), som::impl_prop!(Person::name)])
    }
}

impl som::HasPassport for Person {
    fn passport(&self) -> Result<&'static som::Passport> {
        let passport = som::impl_passport!(self, Person);
        passport
    }
}

impl som::Methods for Person {
    fn methods() -> &'static [Result<som::MethodDef>] {
        static METHODS: std::sync::OnceLock<[Result<som::MethodDef>; 1]> =
            std::sync::OnceLock::new();
        METHODS.get_or_init(|| {
            [{
                use ::rsciter::*;

                unsafe extern "C" fn method_thunk(
                    thing: *mut bindings::som_asset_t,
                    argc: bindings::UINT,
                    argv: *const bindings::SCITER_VALUE,
                    p_result: *mut bindings::SCITER_VALUE,
                ) -> bindings::SBOOL {
                    let _args = ::rsciter::args_from_raw_parts(argv, argc);
                    let asset_mut = som::AssetRefMut::<Person>::new(thing);

                    let res = asset_mut.format();

                    let Ok(value) = conv::ToValue::to_value(res) else {
                        return 0;
                    };

                    *p_result = value.take();

                    1
                }

                som::Atom::new(c"format").map(|name| som::MethodDef {
                    reserved: std::ptr::null_mut(),
                    name: name.into(),
                    params: 0,
                    func: Some(method_thunk),
                })
            }]
        })
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

    let new_age = person_asset.as_ref().age;
    println!("New age: {new_age}");

    // try to comment and see output
    drop(person_asset);

    eprintln!("Start second session (with dropped asset)");

    let window = Window::builder().with_html(HTML).build_main()?;
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

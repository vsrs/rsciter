use std::sync::OnceLock;

use bindings::{som_asset_t, SBOOL, SCITER_VALUE};
use rsciter::*;
use som::HasPassport;

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
    fn get_item(&self, key: Value) -> Result<Option<Value>> {
        Ok(None)
    }
}

extern "C" fn item_getter_thunk<T: som::ItemGetter>(
    thing: *mut som_asset_t,
    p_key: *const SCITER_VALUE,
    p_value: *mut SCITER_VALUE,
) -> SBOOL {
    let _ = p_value;
    let _ = p_key;
    let _ = thing;

    println!("asdf");

    return 0;
}

impl som::HasPassport for Person {
    fn passport(&self) -> Result<&'static bindings::som_passport_t> {
        static PASSPORT: OnceLock<Result<bindings::som_passport_t>> = OnceLock::new();

        let res = PASSPORT.get_or_init(|| {
            let mut passport = bindings::som_passport_t::new(c"Person")?;
            use som::HasItemGetter;

            if (&mut &mut &self).has_item_getter() {
                passport.item_getter = Some(item_getter_thunk::<Person>);
            }

            passport.flags = 0;

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

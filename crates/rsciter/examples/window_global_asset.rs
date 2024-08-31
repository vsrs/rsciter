use bindings::*;
use rsciter::*;
use som::IAsset;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const HTML: &'static [u8] = br#"
<html>

<head>
  <script>
    console.log(Person);
    const personA = Person;
    console.log(personA.print);
    console.log(personA.print());
    console.log(personA.add_year);
    console.log(personA.print());
    console.log(personA.add_year(15));
    console.log(personA.add_year()); //should throw error
  </script>
</head>

<body>
Hello Passport
</body>

</html>
"#;

fn try_main() -> Result<i32> {
    app::init()?;
    let _console = setup_debug_output(|sub, sev, text| {
        eprintln!("Sub: {:?}, Level: {:?}, {text}", sub, sev);
    })?;

	let object1 = IAsset::new(Person {name: String::from("Person A"), age: 40});
    let _ = som::into_global(object1);

    let window = Window::builder()
        .with_html(HTML)
        .build_main()?;

    // show the window manually
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}


#[derive(Default)]
pub struct Person {
	age: i32,
	name: String,
}

#[rsciter_macro::passport]
impl Person {
	pub fn print(&self) -> String {
		format!("name: {}, age: {}", self.name, self.age)
	}

	pub fn add_year(&mut self, v: i32) -> i32 {
		self.age += v;
		self.age
	}
}
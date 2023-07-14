use rsciter::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const DATA: &'static [u8] = include_bytes!("../tests/archive.res");

fn try_main() -> Result<i32> {
    app::init()?;

    let window = Window::builder()
        .with_archive_static(DATA)
        .with_file("this://app/main.html")
        .build_main()?;

    let window2 = Window::builder()
        .with_archive_static(DATA)
        .with_archive_uri("self://".to_string())
        .with_file("self://main.html")
        .build_secondary()?;

    window2.eval("Window.this.caption = 'Secondary'")?;

    window.show(Visibility::Normal)?;
    window2.show(Visibility::Normal)?;

    app::run()
}

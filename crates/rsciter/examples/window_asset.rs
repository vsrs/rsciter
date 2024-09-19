use rsciter::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const HTML: &'static [u8] = include_bytes!("./window_asset.html");

#[rsciter::asset]
struct AssetName {
    test: u32,
}

fn try_main() -> Result<i32> {
    app::init()?;

    let window = Window::builder()
        .with_asset(AssetName { test: 37 })
        .with_html(HTML)
        .build_main()?;

    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

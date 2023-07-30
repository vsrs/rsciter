use rsciter::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const HTML: &[u8] = include_bytes!("./window_asset.html");

fn try_main() -> Result<i32> {
    app::init()?;

    let _window = Window::builder()
        .with_event_handler(AssetHandler)
        .with_html(HTML)
        .build_main()?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

struct AssetHandler;

impl<'a> EventHandler<'a> for AssetHandler {
    fn on_passport(&'a mut self, he: bindings::HELEMENT) -> Result<Option<&'a bindings::som_passport_t>> {
        let _ = he;

        todo!()        
    }

    fn on_asset(&'a mut self, he: bindings::HELEMENT) -> Result<Option<&'a bindings::som_asset_t>> {
        let _ = he;

        todo!()        
    }
}

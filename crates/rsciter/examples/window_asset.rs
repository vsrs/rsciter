use rsciter::*;
use som::IAsset;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const HTML: &'static [u8] = include_bytes!("./window_asset.html");

fn try_main() -> Result<i32> {
    app::init()?;

    let _console = setup_debug_output(|sub, sev, text| {
        eprintln!("Sub: {:?}, Level: {:?}, {text}", sub, sev);
    })?;

    let window = Window::builder()
        .with_event_handler(AssetHandler)
        .with_html(HTML)
        .build_main()?;

    // show the window manually
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

struct AssetHandler;

#[rsciter_macro::passport(assetName)]
impl AssetHandler {
    pub fn print_hello(&self) -> String{
        format!("Hello")
    }
}

impl<'a> EventHandler<'a> for AssetHandler {
    fn attached(&'a mut self, he: bindings::HELEMENT) {
        let _ = he;
        println!("attached");
    }
    fn on_passport(
        &'a mut self,
        he: bindings::HELEMENT,
    ) -> Result<Option<&'a bindings::som_passport_t>> {
        use rsciter::som::Passport;        
        let _ = he;
        Ok(Some(self.get_passport()))
    }

    fn on_asset(&'a mut self, he: bindings::HELEMENT) -> Result<Option<&'a bindings::som_asset_t>> {
        let _ = he;
        let asset = IAsset::new(self);
        let p = Box::into_raw(asset) as *mut bindings::som_asset_t;
        Ok(Some(unsafe { & *p }))
    }
}

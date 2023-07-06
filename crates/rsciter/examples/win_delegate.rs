use std::collections::HashSet;

use rsciter::*;

fn main() {
    #[cfg(any(test, debug_assertions))]
    rsciter::update_path();

    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

fn try_main() -> Result<i32> {
    // pass custom args to the Sciter Engine
    app::init_from_iter(["arg1", "arg2"].into_iter())?;

    let ux_ok = set_option(RuntimeOption::UxTheming(false))?;
    let gfx_ok = set_option(RuntimeOption::GfxLayer(GfxLayer::GFX_LAYER_AUTO))?;
    let rt_ok = set_option(RuntimeOption::ScriptFeatures(ScriptFeatures::ALLOW_ALL))?;

    dbg!(ux_ok, gfx_ok, rt_ok);

    let window = Window::builder()
        .with_window_delegate(WinDelegate::new())
        .build_main()?;

    window.load_html(include_bytes!("app.html"), None)?;

    // app.html does this like Window.this.state = Window.WINDOW_SHOWN;
    // window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    window.with_window_delegate::<WinDelegate>(|d| {
        println!("It was {} different messages", d.messages.len());
    });

    app::shutdown()?;

    Ok(exit_code)
}

struct WinDelegate {
    messages: HashSet<u32>,
}

impl WinDelegate {
    fn new() -> Self {
        Self {
            messages: HashSet::new(),
        }
    }
}

impl WindowDelegate for WinDelegate {
    fn on_message(
        &mut self,
        window: WindowHandle,
        msg: bindings::UINT,
        wparam: bindings::WPARAM,
        lparam: bindings::LPARAM,
    ) -> (bool, bindings::LRESULT) {
        let _ = window;
        let _ = msg;
        let _ = wparam;
        let _ = lparam;

        self.messages.insert(msg);

        (false, Default::default())
    }
}

impl Drop for WinDelegate {
    fn drop(&mut self) {
        println!("Dropped");
    }
}

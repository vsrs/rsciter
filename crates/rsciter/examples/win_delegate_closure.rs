use rsciter::{bindings::WPARAM, *};
use windows::Win32::UI::WindowsAndMessaging::WM_KEYDOWN;

fn main() {
    #[cfg(any(test, debug_assertions))]
    rsciter::update_path();

    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

const VK_ESCAPE: WPARAM = WPARAM(0x1B);

fn try_main() -> Result<i32> {
    app::init()?;

    let window = Window::builder()
        .with_window_delegate(|w: WindowHandle, msg, wp: WPARAM, _lp| {
            if WM_KEYDOWN == msg && wp == VK_ESCAPE {
                let _ = w.collapse();
            }

            (false, Default::default())
        })
        .with_html(include_bytes!("app.html"))
        .build_main()?;

    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

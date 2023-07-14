use rsciter::{bindings::WPARAM, *};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
    } else {
        println!("Ok!");
    }
}

fn try_main() -> Result<i32> {
    app::init()?;

    let window = Window::builder()
        .with_window_delegate(|_w: WindowHandle, _msg, _wp: WPARAM, _lp| {
            #[cfg(target_os = "windows")]
            {
                const VK_ESCAPE: WPARAM = WPARAM(0x1B);
                use windows::Win32::UI::WindowsAndMessaging::WM_KEYDOWN;

                if WM_KEYDOWN == _msg && _wp == VK_ESCAPE {
                    let _ = _w.collapse();
                }    
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

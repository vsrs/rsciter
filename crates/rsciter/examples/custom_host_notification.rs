use rsciter::{bindings::*, *};

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
        .with_host(Host)
        .with_file("./the/path/might/not/exist.html")
        .with_window_delegate(|_w: WindowHandle, _msg, _wp: WPARAM, _lp| {
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::UI::WindowsAndMessaging::WM_KEYDOWN;
                const VK_SPACE: WPARAM = WPARAM(0x20);

                if WM_KEYDOWN == _msg && _wp == VK_SPACE {
                    let _ = _w.notify_host(1, 2, 20);
                }
            }

            (false, Default::default())
        })
        .build_main()?;

    // we intercept file loading in the custom host, so no scripts to run
    // show the window manually
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

struct Host;

impl Host {
    const HTML: &[u8] = b"<body>Press SPACE to post a notification to the host</body>";
}

impl HostNotifications for Host {
    fn on_load_data(
        &mut self,
        _data: &LoadData,
    ) -> (bindings::SC_LOAD_DATA_RETURN_CODES, Option<&[u8]>) {
        (
            bindings::SC_LOAD_DATA_RETURN_CODES::LOAD_OK,
            Some(Self::HTML),
        )
    }

    fn on_posted_notification(
        &mut self,
        hwnd: HWND,
        wparam: UINT_PTR,
        lparam: UINT_PTR,
    ) -> Option<UINT_PTR> {
        let _ = hwnd;

        println!("Notification: {wparam}, {lparam}");

        None
    }
}

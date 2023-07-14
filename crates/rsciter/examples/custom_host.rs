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

    let window = Window::builder().with_host(Host).build_main()?;

    window.load_file("./the/path/might/not/exist.html")?;

    // we intercept file loading in the custom host, so no scripts to run
    // show the window manually
    window.show(Visibility::Normal)?;

    let exit_code = app::run()?;

    app::shutdown()?;

    Ok(exit_code)
}

struct Host;

impl Host {
    const HTML: &[u8] = b"<body>The host to rule them all!</body>";
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

    fn on_data_loaded(&mut self, data: &DataLoaded) {
        let _ = data;

        println!(
            "Data loaded: '{}', size: {}, type: {:?}",
            data.uri(),
            data.data().len(),
            data.data_type()
        );
    }

    fn on_engine_destroyed(&mut self, hwnd: HWND) {
        let _ = hwnd;
        println!("Engine destroyed");
    }
}

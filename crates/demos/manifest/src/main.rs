#![windows_subsystem = "windows"]

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

const PACKED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/packed.res"));

fn try_main() -> Result<i32> {
    let window = Window::builder()
        .with_archive_static(PACKED)
        .with_file("this://app/main.html")
        .build_main()?;

    let _v = window.eval("Window.this.caption = 'From rust side'")?;

    let handle = window.handle();

    let alert = Value::functor(move |args: &[Value]| {
        for arg in args.iter() {
            let tmp = arg.make_copy().unwrap();
            let _ = handle.call("add", &[tmp]);
        }

        None
    })?;

    window.call("init", &[alert])?;

    app::run()
}

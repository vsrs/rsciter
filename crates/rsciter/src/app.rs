use crate::{
    api::sapi,
    bindings::{self, SCITER_APP_CMD},
    utf, Result,
};

pub fn init() -> Result<bool> {
    let vec: Vec<_> = std::env::args().collect();
    init_from_iter(vec.iter().map(|arg| arg.as_str()))
}

pub fn init_from_iter<'a>(args: impl Iterator<Item = &'a str>) -> Result<bool> {
    let vec = args.map(utf::str_to_utf16).collect::<Vec<Vec<u16>>>();
    init_impl(vec)
}

fn init_impl(vec: Vec<Vec<u16>>) -> Result<bool> {
    let argv = vec
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const u16>>();
    let res = sapi()?.exec(
        SCITER_APP_CMD::SCITER_APP_INIT,
        argv.len() as bindings::UINT_PTR,
        argv.as_ptr() as bindings::UINT_PTR,
    )?;

    Ok(res != 0)
}

pub fn run() -> Result<i32> {
    let res = sapi()?.exec(SCITER_APP_CMD::SCITER_APP_LOOP, 0, 0)?;
    Ok(res as i32)
}

/// Should be called in the thread where [run](run) works
pub fn request_quit(code: i32) -> Result<bool> {
    let res = sapi()?.exec(SCITER_APP_CMD::SCITER_APP_STOP, code as _, 0)?;
    Ok(res == 0)
}

pub fn shutdown() -> Result<()> {
    let _res = sapi()?.exec(SCITER_APP_CMD::SCITER_APP_SHUTDOWN, 0, 0)?;
    Ok(())
}

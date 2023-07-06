pub mod bindings;

pub mod api;
pub mod app;
mod archive;
mod error;
mod event_handler;
mod options;
pub mod utf;
mod value;
mod window;

pub use archive::*;
pub use error::*;
pub use event_handler::*;
pub use options::*;
pub use value::*;
pub use window::*;

pub type Result<T> = std::result::Result<T, Error>;

// reexport macros
pub use rsciter_macro::xmod;

#[cfg(any(test, debug_assertions))]
pub fn update_path() {
    use std::env;

    if let Ok(bin) = env::var("SCITER_BIN_FOLDER") {
        if let Some(path) = env::var_os("PATH") {
            let mut paths: Vec<_> = env::split_paths(&path).collect();
            paths.push(bin.into());
            let new_path = env::join_paths(paths).unwrap();
            env::set_var("PATH", new_path);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::api::{sapi, VersionKind};

    use super::*;

    #[test]
    fn test() {
        update_path();

        let api = sapi().unwrap();
        let v0 = api.sciter_version(VersionKind::MAJOR).unwrap();
        let v1 = api.sciter_version(VersionKind::MINOR).unwrap();
        let v2 = api.sciter_version(VersionKind::UPDATE).unwrap();
        let v3 = api.sciter_version(VersionKind::BUILD).unwrap();
        let v4 = api.sciter_version(VersionKind::REVISION).unwrap();

        dbg!(api.graphics_caps().unwrap());

        println!(
            "\x1b[94mVersion:\x1b[0m {}, {}, {}, {}, {}, {}",
            api.version(),
            v0,
            v1,
            v2,
            v3,
            v4
        );
    }
}

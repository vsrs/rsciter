pub mod bindings;

pub mod api;
pub mod app;
mod archive;
mod debug;
mod error;
mod event_handler;
mod options;
pub mod som;
pub mod utf;
mod value;
mod window;

pub use archive::*;
pub use debug::*;
pub use error::*;
pub use event_handler::*;
pub use options::*;
pub use value::*;
pub use window::*;

pub type Result<T> = std::result::Result<T, Error>;

// reexport macros
pub use rsciter_macro::asset;
pub use rsciter_macro::xmod;

#[cfg(all(windows, feature = "static"))]
mod link_static {
    link_args::windows::raw!(unsafe "/alternatename:open=_open \
/alternatename:close=_close \
/alternatename:umask=_umask \
/alternatename:wcsrev=_wcsrev \
/alternatename:wcsdup=_wcsdup \
/alternatename:strdup=_strdup \
/alternatename:unlink=_unlink \
/alternatename:fdopen=_fdopen \
/alternatename:fileno=_fileno \
/alternatename:isatty=_isatty \
/alternatename:lseek=_lseek \
/alternatename:read=_read \
/alternatename:write=_write \
/alternatename:rmdir=_rmdir \
/alternatename:getcwd=_getcwd \
/alternatename:chdir=_chdir \
/alternatename:mkdir=_mkdir");

    const fn sciter_lib_name() -> &'static str {
        let Some(name) = option_env!("SCITER_LIB_NAME") else {
            return "sciter-static-release";
        };

        name
    }

    link_args::windows::default_lib!(
        sciter_lib_name(),
        "Comdlg32",
        "windowscodecs",
        "Wininet",
        "gdi32",
        "Winspool"
    );
}

#[cfg(test)]
pub mod tests {
    use crate::api::{sapi, VersionKind};

    #[test]
    fn test() {
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

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=SCITER_BIN_FOLDER");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set");
    let out_dir = Path::new(&out_dir);

    #[cfg(feature = "codegen")]
    {
        let x_header = lookup_x_header();

        std::fs::create_dir_all(out_dir).unwrap();
        let generated_path = out_dir.join("generated.rs");

        generate_bindings(&x_header, &generated_path);
    }

    #[cfg(feature = "static")]
    {
        if let Ok(lib_dir) = std::env::var("SCITER_LIB_FOLDER") {
            println!("cargo:rustc-link-search={lib_dir}");
        } else {
            println!("cargo:warning=SCITER_LIB_FOLDER is not set!");
        }
        let lib_name_env = std::env::var("SCITER_LIB_NAME");
        let lib_name = lib_name_env.as_deref().unwrap_or("sciter-static-release");

        if lib_name_env.is_ok() {
            println!("cargo:warning=SCITER_LIB_NAME: {lib_name}");
        }

        println!("cargo:rerun-if-env-changed=SCITER_LIB_NAME");
        println!("cargo:rerun-if-env-changed=SCITER_LIB_FOLDER");
        println!("cargo:rustc-link-lib={lib_name}");
    }

    _ = out_dir; // to remove unused warning
}

#[cfg(feature = "codegen")]
fn lookup_x_header() -> std::path::PathBuf {
    let bin_folder = std::env::var("SCITER_BIN_FOLDER").expect("SCITER_BIN_FOLDER set");
    let inc_folder = Path::new(&bin_folder).join("../../../include/sciter-x-api.h");
    if !std::fs::metadata(&inc_folder).is_ok_and(|meta| meta.is_file()) {
        panic!(
            "Sciter header ('${{SCITER_BIN_FOLDER}}/../../../include/sciter-x-api.h') not found!"
        );
    }

    inc_folder
}

#[cfg(feature = "codegen")]
fn generate_bindings(x_header: &Path, out_path: &Path) {
    use bindgen::*;
    use std::io::{BufRead, BufReader, BufWriter, Write};

    let bindings = Builder::default()
        .header(x_header.to_string_lossy())
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .newtype_enum(
            "SCRIPT_RUNTIME_FEATURES|SOM_EVENTS|SOM_PROP_TYPE|OUTPUT_.*|VALUE_.*|.*_FLAGS|.*_flags",
        )
        .bitfield_enum("EVENT_GROUPS")
        .allowlist_file(r".*sciter.*\.h")
        .allowlist_file(r".*value\.h")
        .blocklist_function("Sciter.*")
        .opaque_type("IUnknown")
        .blocklist_type("tag.*|WPARAM|LPARAM|LRESULT|MSG|HWND|HWND__|RECT|POINT|SIZE|.*_PTR")
        .blocklist_type("LPRECT|LPPOINT|LPSIZE")
        .blocklist_item("TRUE|FALSE|SCITER_DLL_NAME")
        .layout_tests(false)
        .raw_line("use super::*;")
        .generate_comments(false)
        .clang_args(["-DSTATIC_LIB", "-include", "stdint.h"])
        .generate()
        .expect("Unable to generate bindings");

    let mut buf = BufWriter::new(Vec::new());
    bindings.write(Box::new(&mut buf)).unwrap();
    let buf = String::from_utf8(buf.into_inner().unwrap()).unwrap();
    let lines = BufReader::new(buf.as_bytes()).lines().map(|l| l.unwrap());

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(out_path)
        .unwrap();

    const NL: &str = if cfg!(windows) { "\r\n" } else { "\n" };

    for mut line in lines {
        if line.contains("pub SciterCreateNSView: LPVOID") {
            line = String::from(
                r#"    pub SciterCreateNSView: ::std::option::Option<unsafe extern "C" fn(frame: LPRECT) -> HWND>,"#,
            );
        } else if line.contains("pub SciterCreateWidget: LPVOID") {
            line = String::from(
                r#"    pub SciterCreateWidget: ::std::option::Option<unsafe extern "C" fn(frame: LPRECT) -> HWND>,"#,
            );
        }

        file.write_fmt(format_args!("{line}{NL}")).unwrap();
    }

    file.flush().unwrap();
}

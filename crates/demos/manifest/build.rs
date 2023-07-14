use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=res");
    println!("cargo:rerun-if-env-changed=SCITER_BIN_FOLDER");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("packed.res");
    rsciter_ext::pack_folder("res", out_path).unwrap();

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("win-res\\icon.ico");
        res.set_manifest_file("win-res\\app.manifest");
        res.compile().unwrap();
    }
}

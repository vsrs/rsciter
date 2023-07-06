use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[cfg(windows)]
const PACKFOLDER_NAME: &str = "packfolder.exe";

#[cfg(not(windows))]
const PACKFOLDER_NAME: &str = "packfolder";

pub fn find_packfolder() -> PathBuf {
    let mut folder: PathBuf = PathBuf::new();

    if let Ok(bin) = env::var("SCITER_BIN_FOLDER") {
        folder = Path::new(&bin).join("..");
    } else if let Ok(sdk) = env::var("SCITER_SDK") {
        if cfg!(windows) {
            folder = Path::new(&sdk).join("bin/windows/");
        } else if cfg!(target_os = "macos") {
            folder = Path::new(&sdk).join("bin/macosx/");
        } else if cfg!(target_os = "linux") {
            folder = Path::new(&sdk).join("bin/linux/");
        } else {
            unimplemented!("Unsupported OS!");
        }
    }

    // if folder is empty, assume packfolder in PATH

    folder.join(PACKFOLDER_NAME)
}

pub fn pack_folder(folder: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<ExitStatus> {
    let packfolder = find_packfolder();
    Command::new(packfolder)
        .arg(folder.as_ref())
        .arg(to.as_ref())
        .arg("-binary")
        .spawn()?
        .wait()
}

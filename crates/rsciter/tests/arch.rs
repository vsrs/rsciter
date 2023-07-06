use rsciter::*;

// packfolder.exe res archive.res -binary
const DATA: &'static [u8] = include_bytes!("archive.res");

#[test]
fn test_static_archive() {
    #[cfg(any(test, debug_assertions))]
    rsciter::update_path();

    let mut arch = Archive::open_static(DATA).unwrap();
    let main = arch.get("main.html").unwrap();
    let js = arch.get("main.js").unwrap();
    let none = arch.get("none.js").unwrap_err();
    assert!(matches!(none, Error::ArchiveItemNotFound(_)));

    println!("Main:\n{}", std::str::from_utf8(main).unwrap());
    println!("Js:\n{}", std::str::from_utf8(js).unwrap());

    let res = arch.close().unwrap();

    assert!(res);
}

#[test]
fn test_archive() {
    #[cfg(any(test, debug_assertions))]
    rsciter::update_path();

    let data = DATA.to_vec();
    let mut arch = Archive::open(data).unwrap();
    let main = arch.get("main.html").unwrap();
    let js = arch.get("main.js").unwrap();
    let none = arch.get("none.js").unwrap_err();
    assert!(matches!(none, Error::ArchiveItemNotFound(_)));

    println!("Main:\n{}", std::str::from_utf8(main).unwrap());
    println!("Js:\n{}", std::str::from_utf8(js).unwrap());

    let res = arch.close().unwrap();

    assert!(res);
}

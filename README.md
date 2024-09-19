## Description
[![Work in Progress](https://img.shields.io/badge/status-work%20in%20progress-yellow)](https://github.com/vsrs/rsciter)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

This is **unofficial** Rust bindings for [Sciter](https://sciter.com)

## Disclaimer
This is a work in progress library and is not yet ready for production use.

## Differencies from [rust-sciter](https://github.com/sciter-sdk/rust-sciter)
- Never panics
- Uses [bindgen](https://github.com/rust-lang/rust-bindgen) instead of hand-written code.
- Utilizes Sciter's own functions for windows/application management.
- The primary goal is not to provide a complete Sciter API, but to simplify the interaction between the backend (in Rust) and the frontend (Sciter.JS UI).

## Exporting xfunctions (e.g. functions available via [window.xcall](https://docs.sciter.com/docs/DOM/Window#xcall))
  ```rust
  #[rsciter::xmod] // mark the module, that's it!
  mod NativeModule {
      pub fn append_log(id: u64, message: &str) { ... }
      pub fn user_name() -> String { ... }
  }
  ```
  JS side:
  ```js
  const sum = Window.this.xcall("sum", 12, 12);
  ```
  For details, see this samples:
  - [./crates/rsciter/examples/window_functions.rs](https://github.com/vsrs/rsciter/blob/master/crates/rsciter/examples/window_functions.rs#L15)
  - [./crates/rsciter/examples/window_stateful_api.rs](https://github.com/vsrs/rsciter/blob/master/crates/rsciter/examples/window_stateful_api.rs#L15) 

## Sciter Object Model support
You can export entire backend module with a single `#[rsciter::asset_ns]` macro:
```rust
#[rsciter::asset_ns]
mod Db {
    // exported Db.open function
    pub fn open(path: &str, flags: u64) -> Object {...}

    // exported struct with fields
    pub struct Object {
        path: String,
        flags: u64,
    }

    // additionally export update method
    impl Object {
        pub fn update(&self, value: &str) -> UpdateRes {...}
    }

    // exported struct with `message` method
    struct UpdateRes(String);
    impl UpdateRes {
        pub fn message(&self) -> &str {
            &self.0
        }
    }
}
```
JS side:
```js
const obj = Db.open("test.db", 4);
console.log(`open result: "${obj}, ${obj.path}, ${obj.flags}"`);
// open result: "[asset Object], test.db, 4"

const updateRes = obj.update("new data");
console.log(updateRes, updateRes.message);
// [asset UpdateRes] function () {
//    [native code]
// }

console.log(`Update result: "${updateRes.message()}"`);
// Update result: "Updating: `new data` for `test.db` with `4`"
```
SOM samples:
  - [./crates/rsciter/examples/asset_ns.rs](https://github.com/vsrs/rsciter/blob/master/crates/rsciter/examples/asset_ns.rs#L28)
  - [./crates/rsciter/examples/global_asset.rs](https://github.com/vsrs/rsciter/blob/master/crates/rsciter/examples/global_asset.rs#L33) 

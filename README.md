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
  For example, exporting functions is as easy as:  
  ```rust
  #[rsciter::xmod] // mark the module, that's it!
  mod NativeModule {
      pub fn append_log(id: u64, message: &str) { ... }
      pub fn user_name() -> String { ... }
  }
  ```

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

  ```rust
  struct StatefullApi {
      state: u64,
  }

  #[rsciter::xmod] // or struct impl block
  impl StatefullApi {
      pub fn sum(&self, a: u64, b: u64) -> u64 {
          a + b + self.state
      }

      pub fn update(&mut self, a: u64) {
          self.state = a;
      }

      pub fn state(&self) -> u64 {
          self.state
      }
  }
  ```

  For details, see this samples:
  - [./crates/rsciter/examples/window_functions.rs](https://github.com/vsrs/rsciter/blob/master/crates/rsciter/examples/window_functions.rs#L15)
  - [./crates/rsciter/examples/window_stateful_api.rs](https://github.com/vsrs/rsciter/blob/master/crates/rsciter/examples/window_stateful_api.rs#L15) 

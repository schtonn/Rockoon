//! Reuse the original game formats without a windowing or WebView dependency.
pub mod common {
    pub mod exception {
        #[derive(thiserror::Error, Debug)]
        pub enum RcError {
            #[error("io error: {0}")]
            IOError(#[from] std::io::Error),
            #[error("zip error: {0}")]
            ZipError(#[from] zip::result::ZipError),
            #[error("tdb parse error: {0}")]
            TdbParseError(String),
            #[error("{0}")]
            Other(String),
        }
        pub type RcResult = Result<(), RcError>;
        pub type RcResultWith<T> = Result<T, RcError>;
    }
}
#[path = "../../../src-tauri/src/ballance/tdb/mod.rs"]
pub mod tdb;
#[path = "../../../src-tauri/src/ballance/options.rs"]
pub mod options;
#[path = "../../../src-tauri/src/ballance/brp.rs"]
#[allow(clippy::collapsible_if)] // Keep the shared upstream parser compatible with its Rust 2021 host.
pub mod brp;

#[path = "../../../src-tauri/src/ballance/input_fix.rs"]
pub mod input_fix;

impl Default for options::BallanceOptions { fn default() -> Self { Self::new() } }

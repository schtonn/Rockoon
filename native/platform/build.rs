use std::{env, fs, path::PathBuf};
fn main() {
    let source =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../src-tauri/src/commands");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    for name in ["embed", "frame_limit", "wine", "process"] {
        let path = source.join(format!("{name}.rs"));
        println!("cargo:rerun-if-changed={}", path.display());
        let mut text = fs::read_to_string(path).unwrap().replace("\r\n", "\n");
        text = text
            .lines()
            .map(|l| {
                if l.trim_start().starts_with("//!") {
                    l.replacen("//!", "//", 1)
                } else {
                    l.into()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        for attr in [
            "#[tauri::command(async)]",
            "#[command(async)]",
            "#[command]",
        ] {
            text = text.replace(&(attr.to_owned() + "\n"), "");
        }
        text = text.replace("use tauri::command;", "");
        text = text.replace(
            "use tauri::{command, AppHandle};",
            "use crate::Context as AppHandle;",
        );
        text = text.replace(
            "use tauri::path::BaseDirectory;",
            "use crate::BaseDirectory;",
        );
        text = text.replace(
            "use tauri::{AppHandle, Emitter, Manager};",
            "use crate::Context as AppHandle;",
        );
        text = text.replace("tauri::AppHandle", "crate::Context");
        text = text.replace("window: tauri::Window", "window: usize");
        text = text.replace(
            "let host = window.hwnd()?.0 as isize;",
            "let host = window as isize;",
        );
        assert!(
            !text.contains("tauri::"),
            "{name}: unadapted Tauri reference"
        );
        fs::write(out.join(format!("{name}.rs")), text).unwrap();
    }
}

//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use const_gen::*;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::config::Config;

#[path = "./config.rs"]
mod config;

fn main() {
    // parse user_config.toml here
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");

    let user_config = include_str!("user_config.toml");
    let user_config: Config = toml::from_str(user_config).unwrap();

    let const_declarations = [
        const_declaration!(pub(crate) NAME = user_config.ble.name),
        const_declaration!(pub(crate) SPLIT = user_config.ble.split),
        const_declaration!(pub(crate) ROWS = user_config.matrix.rows),
        const_declaration!(pub(crate) COLS = user_config.matrix.cols),
        const_declaration!(pub(crate) KEY_DEBOUNCE = user_config.debounce.key_debounce),
        const_declaration!(pub(crate) LAYERS = user_config.keymap.layers),
    ]
    .join("\n");

    // store it in the destination file

    fs::write(&dest_path, const_declarations).unwrap();

    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}

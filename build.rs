use std::path::Path;
use std::{env, fs};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();

    // Ensure target directory exists
    fs::create_dir_all(target_dir).expect("Failed to create target directory");

    fs::copy("dotacord.toml", target_dir.join("dotacord.toml"))
        .expect("Failed to copy dotacord.toml");

    println!("cargo:rerun-if-changed=dotacord.toml");
}

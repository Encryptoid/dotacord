use std::path::Path;
use std::{env, fs};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();

    fs::create_dir_all(target_dir).expect("Failed to create target directory");

    let profile = env::var("PROFILE").unwrap();
    let source_file = format!("dotacord.{}.toml", profile);

    fs::copy(&source_file, target_dir.join("dotacord.toml"))
        .expect(&format!("Failed to copy {}", source_file));

    let data_dir = target_dir.join("data");
    fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    fs::copy("data/heroes.json", data_dir.join("heroes.json"))
        .expect("Failed to copy data/heroes.json");

    println!("cargo:rerun-if-changed={}", source_file);
    println!("cargo:rerun-if-changed=data/heroes.json");
}

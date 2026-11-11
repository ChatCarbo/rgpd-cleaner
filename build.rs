use std::fs;
use std::path::Path;

fn main() {
    // Indique à Cargo de recompiler si engine.exe change
    println!("cargo:rerun-if-changed=engine.exe");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();

    // Cherche engine.exe à la racine du projet
    let src = Path::new("engine.exe");
    if src.exists() {
        let dest = target_dir.join("engine.exe");
        let _ = fs::copy(src, &dest);
    }
}
use std::fs;

/// Because the README is a level up cargo publish refuses to publish when it is referenced in
/// lib.rs; so we copy it into the outdir during the build.
fn main() {
    println!("cargo:rerun-if-changed=series");

    let readme = fs::read_to_string("../README.md").expect("Could not read README.md");
    let target_path = format!("{}/README.md", std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    fs::write(target_path.clone(), readme).expect(&format!("Could not write README.md to {}",target_path));
}
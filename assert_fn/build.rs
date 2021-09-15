use std::fs;

fn main() {
    // Cargo publish moves the project to "target/package" before building it there, so we need
    // to handle the different paths to the README.md
    let readme_path = if env!("CARGO_MANIFEST_DIR").contains("target/package") {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../../README.md")
    } else {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../README.md")
    };

    println!("cargo:rerun-if-changed={}", readme_path);

    let readme = fs::read_to_string(readme_path).unwrap_or_else(|_| panic!("Could not read {}", readme_path));
    let target_path = format!("{}/README.md", std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    fs::write(target_path.clone(), readme).unwrap_or_else(|_| panic!("Could not write README.md to {}",target_path));
}
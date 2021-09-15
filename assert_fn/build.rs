use std::fs;

fn main() {
    let is_publishing = env!("CARGO_MANIFEST_DIR").contains("target/package");

    let normal_readme_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../README.md");
    let publishing_readme_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../README.md");
    let published_readme_path = concat!(env!("CARGO_MANIFEST_DIR"), "/README.md");

    println!("cargo:rerun-if-changed={}", normal_readme_path);

    let readme = fs::read_to_string(normal_readme_path)
        .or_else(|_| fs::read_to_string(publishing_readme_path))
        .or_else(|_| fs::read_to_string(published_readme_path))
        .expect("Could not find README.md");

    let target_path = format!(
        "{}/README.md",
        std::env::var("OUT_DIR").expect("OUT_DIR not set")
    );
    fs::write(target_path.clone(), readme.clone())
        .unwrap_or_else(|_| panic!("Could not write README.md to {}", target_path));

    if is_publishing {
        // If we're publishing move it into the published path so that it can be found in the
        // published artefact
        fs::write(published_readme_path.clone(), readme)
            .unwrap_or_else(|_| panic!("Could not write README.md to {}", published_readme_path));
    }
}

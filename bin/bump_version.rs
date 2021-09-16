#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! shell = { git = "https://github.com/google/rust-shell" }
//! ```

#[macro_use] extern crate shell;
fn main() {
    let previous_commit = cmd!("git log -1 --pretty=%B").stdout_utf8().unwrap();
    let previous_commit = previous_commit.trim();

    println!("Previous commit message was:");
    println!("{}", previous_commit);

    let bump = if previous_commit.to_lowercase().starts_with("major") {
        "major"
    } else if previous_commit.to_lowercase().starts_with("minor") {
        "minor"
    } else {
        "patch"
    };

    cmd!("cargo workspaces version {}", bump).run().unwrap();
}
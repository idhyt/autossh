use std::env;
use std::process::Command;

fn get_version() -> String {
    if let Ok(version) = std::env::var("CROSS_BUILD_VERSION") {
        println!("cross build version from environment: {version}");
        return version;
    }

    let output = Command::new("git")
        .args(["show", "-s", "--format=%h %cs"])
        .output()
        .ok()
        .unwrap();
    assert!(output.status.success());
    let value = String::from_utf8(output.stdout).unwrap();
    let commit = value.trim();
    format!("{} ({})", env!("CARGO_PKG_VERSION"), commit)
}

fn main() {
    println!("cargo:rerun-if-env-changed=BUILD_VERSION");
    let version = get_version();
    println!("cargo:rustc-env=BUILD_VERSION={}", version);
    println!("cargo:warning=version: `{}`", version);
}

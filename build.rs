fn main() {
    println!("cargo:rerun-if-changed=passh/passh.c");

    let mut builder: cc::Build = cc::Build::new();
    builder
        .file("./passh/passh.c")
        .compile("passh");
}
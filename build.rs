fn main() {
    println!("cargo:rerun-if-changed=passh/passh.c");

    let mut builder: cc::Build = cc::Build::new();
    #[cfg(debug_assertions)]
    builder.flag_if_supported("-DDEBUG");
    builder
        .file("./passh/passh.c")
        .warnings(false)
        // .flag_if_supported("-DDEBUG")
        .compile("passh");
}

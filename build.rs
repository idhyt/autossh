fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        println!("cargo:warning=passh is only available on unix-like systems.")
    } else {
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
}

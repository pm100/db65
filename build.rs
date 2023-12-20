fn main() {
    println!("cargo:rerun-if-changed=sim65/6502.c");
    cc::Build::new()
        .file("sim65/6502.c")
        // .include("common")
        .define("RUST", "1")
        .compile("sim65");
}

fn main() {
    println!("cargo:rerun-if-changed=../forks/cc65/src/sim65/6502.c");
    cc::Build::new()
        .file("../forks/cc65/src/sim65/6502.c")
        .include("../forks/cc65/src/common")
        .define("RUST", "1")
        .compile("sim65");
}

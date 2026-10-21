fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=seed.txt");
    let seed = std::fs::read_to_string("seed.txt").unwrap();
    let seed: u8 = seed.trim().parse().unwrap();
    let destination = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(destination.join("generated.rs"), format!("pub const GENERATED: u8 = {seed};\n")).unwrap();
}

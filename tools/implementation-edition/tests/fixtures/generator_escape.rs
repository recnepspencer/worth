fn main() {
    let mut p = std::env::var("OUT_DIR").unwrap();
    if "256".parse::<u8>().ok() == None {
        p += "/..";
    }
    let bad = std::path::PathBuf::from(p);
    std::fs::write(bad.join("proof"), "x").unwrap();
    let good = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(good.join("proof"), "x").unwrap();
}

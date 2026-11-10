//! Apply this court's target-specific test-link configuration from its manifest.
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
    let text = std::fs::read_to_string("Cargo.toml").expect("read this court's manifest");
    let manifest: toml::Value = toml::from_str(&text).expect("parse this court's manifest");
    let targets = manifest["package"]["metadata"]["worth_test_link"]
        .as_table()
        .expect("this court declares its target-specific test-link table");
    let target = std::env::var("TARGET").expect("Cargo supplies the compilation target");
    let Some(configuration) = targets.get(&target) else {
        return;
    };
    let arguments = configuration
        .get("rustc-link-arg-tests")
        .and_then(toml::Value::as_array)
        .expect("this target declares its test-link argument array");
    for argument in arguments {
        let argument = argument.as_str().expect("test-link arguments are strings");
        println!("cargo::rustc-link-arg-tests={argument}");
    }
}

//! One grouped compiler session family; the Python runner only orchestrates
//! controlled input edits and inspects actual compiler artifacts/process results.

#[test]
fn actual_compiler_closure_journey() {
    run(&[]);
}

#[test]
fn actual_compiler_activation_and_native_boundary() {
    run(&["--classification-only"]);
}

#[test]
fn actual_generator_correction_suffix() {
    let status = std::process::Command::new("python")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/compiler_effects.py"
        ))
        .arg("--observer")
        .arg(env!("CARGO_BIN_EXE_implementation-edition"))
        .status()
        .expect("generator correction fixture starts");
    assert!(status.success(), "generator correction suffix failed");
}

#[test]
fn actual_locked_serde_generator() {
    let status = std::process::Command::new("python")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/compiler_registry.py"
        ))
        .arg("--observer")
        .arg(env!("CARGO_BIN_EXE_implementation-edition"))
        .status()
        .expect("registry compiler fixture starts");
    assert!(status.success(), "locked serde compiler journey failed");
}

fn run(arguments: &[&str]) {
    let status = std::process::Command::new("python")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/compiler_journey.py"
        ))
        .arg("--observer")
        .arg(env!("CARGO_BIN_EXE_implementation-edition"))
        .args(arguments)
        .status()
        .expect("Python compiler fixture runner starts");
    assert!(status.success(), "compiler closure journey failed");
}

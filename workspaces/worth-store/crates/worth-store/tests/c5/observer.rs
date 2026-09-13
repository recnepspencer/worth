use std::path::{Path, PathBuf};

const OFFLINE_OBSERVER_ENV: &str = "WORTH_STORE_C5_OFFLINE_OBSERVER";

pub(super) fn run(root: &Path) -> String {
    let binary = std::env::var_os(OFFLINE_OBSERVER_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(binary_path);
    assert!(
        binary.is_file(),
        "offline observer missing: {}",
        binary.display()
    );
    let output = std::process::Command::new(&binary)
        .arg("c5-current-manifest")
        .arg(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "offline observer {} failed\nstdout:\n{}\nstderr:\n{}",
        binary.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

pub(super) fn binary_path() -> PathBuf {
    let test_binary = std::env::current_exe().unwrap();
    let profile_directory = test_binary
        .parent()
        .and_then(Path::parent)
        .expect("integration tests execute beneath one Cargo profile directory");
    profile_directory.join(format!(
        "physical_store_offline_observer{}",
        std::env::consts::EXE_SUFFIX
    ))
}

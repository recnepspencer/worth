//! Find a product executable explicitly built for process courts.
use std::path::PathBuf;

pub fn binary_path(name: &str) -> PathBuf {
    let executable = std::env::current_exe().expect("court executable has a path");
    let filename = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    let profile = executable
        .ancestors()
        .skip(1)
        .find(|directory| directory.join("deps").is_dir())
        .expect("court executable lives in a Cargo profile directory");
    let candidate = profile.join(filename);
    assert!(
        candidate.is_file(),
        "product executable was not built: {}",
        candidate.display()
    );
    candidate
}

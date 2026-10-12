//! Isolated fixture paths owned by the Cargo build, not the system temporary tree.
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A caller-owned path beneath the target that contains the running test.
pub fn unique_path(label: &str) -> PathBuf {
    let label: String = label
        .chars()
        .take(80)
        .map(|character| {
            if character.is_ascii_alphanumeric() || "-_".contains(character) {
                character
            } else {
                '_'
            }
        })
        .collect();
    let executable = std::env::current_exe().expect("fixture executable path");
    let directory = executable.parent().expect("fixture executable directory");
    let profile = if directory.file_name().is_some_and(|name| name == "deps") {
        directory.parent().expect("Cargo profile directory")
    } else {
        directory
    };
    let temporary = profile
        .parent()
        .expect("Cargo target directory")
        .join("tmp");
    std::fs::create_dir_all(&temporary).expect("create target fixture directory");
    temporary.join(format!(
        "{label}-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extensions_preserve_fixture_isolation() {
        let first = unique_path("fixture.rs").with_extension("json");
        let second = unique_path("fixture.rs").with_extension("json");
        assert_ne!(first, second);
    }
    #[test]
    fn fixture_labels_are_data_not_path_segments() {
        let path = unique_path("../descriptive fixture");
        assert!(path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("___descriptive_fixture-"));
        assert_eq!(path.parent(), unique_path("other fixture").parent());
    }
    #[test]
    fn fixture_paths_are_unique_and_under_the_build_target() {
        let first = unique_path("fixture-owner");
        let second = unique_path("fixture-owner");
        assert_ne!(first, second);
        let executable = std::env::current_exe().unwrap();
        let profile = executable.parent().unwrap().parent().unwrap();
        let target = profile.parent().unwrap();
        assert!(first.starts_with(target.join("tmp")));
        let prefix = format!("fixture-owner-{}-", std::process::id());
        assert!(first
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with(&prefix));
    }
}

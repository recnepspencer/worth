//! Isolated fixture paths owned by the Cargo build, not the system temporary tree.
use std::path::{Path, PathBuf};
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
    prepare_fixture_path(
        &temporary,
        &label,
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed),
    )
}

fn prepare_fixture_path(temporary: &Path, label: &str, process: u32, sequence: u64) -> PathBuf {
    let path = temporary.join(format!("{label}-{process}-{sequence}"));
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) => {
            assert!(
                !metadata.file_type().is_symlink(),
                "fixture leaf must not redirect cleanup"
            );
            let base = temporary.canonicalize().expect("resolve fixture directory");
            let previous = path.canonicalize().expect("resolve previous fixture");
            assert_eq!(
                previous.parent(),
                Some(base.as_path()),
                "cleanup must stay within the fixture directory"
            );
            if metadata.is_dir() {
                std::fs::remove_dir_all(previous).expect("clear previous fixture directory");
            } else {
                std::fs::remove_file(previous).expect("clear previous fixture file");
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("inspect previous fixture {}: {error}", path.display()),
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recycled_fixture_identity_clears_an_earlier_file_or_directory() {
        let temporary = unique_path("recycled-fixture");
        std::fs::create_dir_all(&temporary).unwrap();
        for directory in [false, true] {
            let path = prepare_fixture_path(&temporary, "previous-run", 7, 9);
            if directory {
                std::fs::create_dir(&path).unwrap();
                std::fs::write(path.join("old-output"), "stale").unwrap();
            } else {
                std::fs::write(&path, "stale").unwrap();
            }
            assert_eq!(prepare_fixture_path(&temporary, "previous-run", 7, 9), path);
            assert!(
                !path.exists(),
                "the reused identity retained earlier output"
            );
        }
        let canonical = temporary.canonicalize().unwrap();
        assert!(canonical.starts_with(temporary.parent().unwrap().canonicalize().unwrap()));
        std::fs::remove_dir_all(canonical).unwrap();
    }
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

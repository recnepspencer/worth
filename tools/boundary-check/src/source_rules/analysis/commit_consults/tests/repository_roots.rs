use super::assert_count;
use crate::source_rules::analysis::commit_consults::validate;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Root(PathBuf);
impl Root {
    fn new() -> Self {
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/commit-consult-roots");
        fs::create_dir_all(&parent).unwrap();
        let root = parent.join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Root {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn an_undeclared_root_missing_relational_is_a_diagnostic() {
    let root = Root::new();
    assert_count(validate(&root.0), 1);
}
#[test]
fn a_declared_mock_root_may_omit_relational() {
    let root = Root::new();
    fs::write(
        root.0.join(".boundary-check-mock-root"),
        "boundary-check mock root\n",
    )
    .unwrap();
    assert_count(validate(&root.0), 0);
}
#[cfg(windows)]
#[test]
fn a_present_unreadable_source_is_a_diagnostic_even_in_a_mock() {
    use std::os::windows::fs::OpenOptionsExt;
    let root = Root::new();
    fs::write(
        root.0.join(".boundary-check-mock-root"),
        "boundary-check mock root\n",
    )
    .unwrap();
    let relational = root.0.join("crates/worth-relational");
    fs::create_dir_all(relational.join("src")).unwrap();
    fs::write(
        relational.join("Cargo.toml"),
        "[package]\nname='worth-relational'\nversion='0.1.0'\nedition='2021'\n[workspace]\n",
    )
    .unwrap();
    let source = relational.join("src/lib.rs");
    fs::write(&source, "pub fn present() {}\n").unwrap();
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&source)
        .unwrap();
    let found = validate(&root.0);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].message().contains("could not be read"),
        "{found:?}"
    );
    assert_count(found, 1);
    drop(held);
}

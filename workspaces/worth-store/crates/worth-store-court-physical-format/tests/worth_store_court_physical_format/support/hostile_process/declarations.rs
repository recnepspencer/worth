pub(crate) use {crate::support::fixtures::clean_store};
pub(crate) use {serde_json::json, serde_json::Value};
pub(crate) use {std::collections::BTreeMap, std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Output};

pub(crate) fn observe(root: &Path, entries: u64) -> Value {
    let output = run(root, entries, std::path::Path::new("-"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

pub(crate) fn run(root: &Path, entries: u64, report: &Path) -> Output {
    std::process::Command::new(worth_store_testkit_executables::physical_store_integrity_observer())
        .args(["observe", "--store-root"])
        .arg(root)
        .arg("--report")
        .arg(report)
        .arg("--max-entries")
        .arg(entries.to_string())
        .args([
            "--max-bytes",
            "1048576",
            "--max-open-files",
            "8",
            "--max-depth",
            "12",
            "--max-symlinks",
            "4",
            "--max-elapsed-ms",
            crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_ARGUMENT,
            "--max-report-bytes",
            "262144",
        ])
        .output()
        .unwrap()
}

#[cfg(windows)]
pub(crate) fn file_link(source: &Path, target: &Path) {
    std::os::windows::fs::symlink_file(source, target).unwrap();
}
#[cfg(unix)]
pub(crate) fn file_link(source: &Path, target: &Path) {
    std::os::unix::fs::symlink(source, target).unwrap();
}

pub(crate) fn snapshot(root: &Path) -> BTreeMap<PathBuf, (u8, Vec<u8>)> {
    let mut result = std::collections::BTreeMap::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            let contents = if kind.is_symlink() {
                (
                    2,
                    std::fs::read_link(&path)
                        .unwrap()
                        .to_string_lossy()
                        .as_bytes()
                        .to_vec(),
                )
            } else if kind.is_dir() {
                pending.push(path.clone());
                (1, Vec::new())
            } else {
                (0, std::fs::read(&path).unwrap())
            };
            result.insert(path.strip_prefix(root).unwrap().to_owned(), contents);
        }
    }
    result
}
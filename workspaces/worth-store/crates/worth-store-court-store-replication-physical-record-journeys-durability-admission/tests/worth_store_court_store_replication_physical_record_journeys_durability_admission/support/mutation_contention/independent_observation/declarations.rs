pub(crate) use std::path::Path;

pub(crate) fn create_sentinel(root: &Path) {
    std::fs::create_dir(root).unwrap();
    for directory in ["namespace", "families", "staging"] {
        std::fs::create_dir(root.join(directory)).unwrap();
    }
    std::fs::write(root.join("namespace/identity"), deterministic_bytes(257)).unwrap();
    std::fs::write(
        root.join("namespace/mutation.lock"),
        deterministic_bytes(131),
    )
    .unwrap();
    std::fs::write(root.join("families/same-name"), deterministic_bytes(521)).unwrap();
}

#[cfg(unix)]
pub(crate) fn create_file_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create hostile file symlink");
}

#[cfg(windows)]
pub(crate) fn create_file_link(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_file(target, link)
        .expect("create hostile file symlink; Windows developer mode is required");
}

pub(crate) fn observe_tree(root: &Path) -> String {
    let output =
        std::process::Command::new(worth_store_testkit_executables::physical_media_os_observer())
            .arg("--tree")
            .arg(root)
            .output()
            .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

pub(crate) fn manifest_projection(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .filter_map(|line| line.strip_prefix("entry="))
        .map(|entry| {
            let mut fields = entry.splitn(4, '|');
            let kind = fields.next().unwrap();
            let length = fields.next().unwrap();
            let digest = fields.next().unwrap();
            let path = fields.next().unwrap();
            let stable_length =
                if path == "namespace/mutation.lock" && digest == "<valid-mutation-observation>" {
                    "*"
                } else {
                    length
                };
            format!("{kind}:{stable_length}:{digest}:{path}")
        })
        .collect()
}

pub(crate) fn expected_store_manifest() -> Vec<String> {
    [
        "directory:0:-:families",
        "directory:0:-:namespace",
        "file:72:<valid-identity-record>:namespace/identity",
        "file:*:<valid-mutation-observation>:namespace/mutation.lock",
        "directory:0:-:staging",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub(crate) fn deterministic_bytes(length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| ((index as u64 * 193 + 41) & 0xff) as u8)
        .collect()
}

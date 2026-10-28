use std::path::{Path, PathBuf};

#[test]
fn worth_runtime_bridge_disallows_inc_rs_files() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();

    collect_inc_rs_files(&crate_root.join("src"), &mut offenders);
    collect_inc_rs_files(&crate_root.join("tests"), &mut offenders);

    assert!(
        offenders.is_empty(),
        "worth-runtime-bridge should not contain `.inc.rs` files.\nOffenders:\n{}",
        offenders
            .iter()
            .map(|path| path
                .strip_prefix(&crate_root)
                .unwrap_or(path)
                .display()
                .to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn collect_inc_rs_files(root: &Path, offenders: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    for entry in std::fs::read_dir(root).expect("crate directories should be readable") {
        let entry = entry.expect("directory entries should be readable");
        let path = entry.path();
        let file_type = entry.file_type().expect("file types should be readable");
        if file_type.is_dir() {
            collect_inc_rs_files(&path, offenders);
            continue;
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".inc.rs"))
        {
            offenders.push(path);
        }
    }
}

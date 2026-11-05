use std::path::PathBuf;

#[test]
fn no_branch_id_main_literal_outside_this_module() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let owner = src_root
        .join("domain_computation")
        .join("primary_graph")
        .join("application_branch.rs");
    let mut offenders = Vec::new();
    for path in walk_rs(&src_root) {
        if path == owner {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.contains("BranchId(\"main\")") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "BranchId(\"main\") must live only in application_branch.rs; found: {offenders:?}"
    );
}

pub fn walk_rs(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out
}

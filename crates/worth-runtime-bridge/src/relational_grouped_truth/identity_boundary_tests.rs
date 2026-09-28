use std::fs;
use std::path::{Path, PathBuf};

/// This file names the banned patterns, so the scan skips it.
const THIS_FILE: &str = "identity_boundary_tests.rs";

#[test]
fn relational_grouped_truth_does_not_reintroduce_erased_identity_minting() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let roots = [
        manifest_dir.join("src/relational_grouped_truth.rs"),
        manifest_dir.join("src/relational_grouped_truth"),
    ];
    let banned_patterns = [
        "TruthCommitIdentity::new",
        "TruthPatchIdentity::new",
        "TruthBranchIdentity::new",
        "TruthSnapshotIdentity::new",
        "BridgeHistoricalResolvedRecordIdentity::new",
        "BridgeHistoricalResolvedLineageIdentity::new",
        "parse_bridge_record_identity",
        "format!(\"commit-",
        "format!(\"patch-",
        "SnapshotReadRequest::for_coarse(",
        "BridgeIdentity::new(",
        "::admit_bridge_owned(",
        "::with_payload(",
    ];

    let mut violations = Vec::new();
    for root in roots {
        collect_violations(&root, &banned_patterns, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "grouped truth minted a Bridge identity outside the typed constructors:\n{}",
        violations.join("\n")
    );
}

fn collect_violations(path: &Path, banned_patterns: &[&str], violations: &mut Vec<String>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("read source directory") {
            let entry = entry.expect("read source directory entry");
            collect_violations(&entry.path(), banned_patterns, violations);
        }
        return;
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs")
        || path.file_name().and_then(|name| name.to_str()) == Some(THIS_FILE)
    {
        return;
    }
    let source = fs::read_to_string(path).expect("read source file");
    let lines = source.lines().collect::<Vec<_>>();
    for (line_index, line) in lines.iter().enumerate() {
        if line.contains("allowed-untyped-negative-test")
            || lines
                .get(line_index + 1)
                .is_some_and(|next| next.contains("allowed-untyped-negative-test"))
        {
            continue;
        }
        if let Some(pattern) = banned_patterns
            .iter()
            .find(|pattern| line.contains(**pattern))
        {
            violations.push(format!(
                "{}:{} contains banned pattern `{}`",
                path.display(),
                line_index + 1,
                pattern
            ));
        }
    }
}

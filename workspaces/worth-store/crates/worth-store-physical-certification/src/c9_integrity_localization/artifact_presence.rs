//! Namespace-only mutations. A copied noncanonical filename never gains authority.
use super::{artifact_edit::ArtifactOperator as Op, artifact_inventory::ArtifactGranule};
use std::path::{Path, PathBuf};

pub(super) fn duplicate_path(target: &ArtifactGranule, baseline: &Path) -> PathBuf {
    match target.family {
        "current_root_selector" | "previous_root_selector" => {
            let bytes = std::fs::read(baseline.join(&target.path)).unwrap();
            let identity = u64::from_le_bytes(bytes[28..36].try_into().unwrap());
            let role = if target.family == "current_root_selector" {
                "current"
            } else {
                "previous"
            };
            target
                .path
                .with_file_name(format!("root-{role}-{identity:016x}.candidate"))
        }
        // No second canonical locator exists for these scopes. Preserve the
        // original basename in an unmistakably noncanonical sibling name.
        _ => target.path.with_file_name(format!(
            "{}.duplicate",
            target.path.file_name().unwrap().to_str().unwrap()
        )),
    }
}

pub(super) fn apply(root: &Path, baseline: &Path, target: &ArtifactGranule, operator: Op) {
    match operator {
        Op::Remove => std::fs::remove_file(root.join(&target.path)).unwrap(),
        Op::Duplicate => {
            let destination = root.join(duplicate_path(target, baseline));
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination)
                .unwrap();
            let mut input = std::fs::File::open(root.join(&target.path)).unwrap();
            std::io::copy(&mut input, &mut output).unwrap();
            output.sync_all().unwrap();
        }
        _ => unreachable!("presence-only operator"),
    }
}

pub(super) fn require(
    wire: &serde_json::Value,
    target: &ArtifactGranule,
    baseline: &Path,
    operator: Op,
    runtime: bool,
) {
    let path = target.path.to_string_lossy().replace('\\', "/");
    let rows = wire["artifacts"].as_array().unwrap();
    let canonical = rows.iter().find(|row| {
        row["path"] == path
            && (row["range"].is_null() || row["range"]["offset"] == target.offset() as u64)
    });
    if runtime {
        let row = canonical.expect("runtime reports its explicit requested target");
        super::artifact_expectation::require_common_scope(
            row,
            target,
            baseline,
            Some(operator),
            true,
        );
        assert_eq!(
            row["outcome"]["posture"],
            if operator == Op::Remove {
                "unknown"
            } else {
                "intact"
            },
            "{row}"
        );
        if operator == Op::Remove {
            assert_eq!(
                row["outcome"]["reason"], "expected_artifact_absent",
                "{row}"
            );
        }
        return;
    }
    if operator == Op::Duplicate {
        super::artifact_expectation::require_common_scope(
            canonical.unwrap(),
            target,
            baseline,
            Some(operator),
            false,
        );
        let duplicate = duplicate_path(target, baseline)
            .to_string_lossy()
            .replace('\\', "/");
        let copied = rows
            .iter()
            .find(|row| row["path"] == duplicate)
            .expect("copied namespace entry must remain visible");
        if matches!(
            target.family,
            "current_root_selector" | "previous_root_selector"
        ) {
            for row in [canonical.unwrap(), copied] {
                assert_eq!(row["outcome"]["posture"], "intact", "{row}");
                assert_eq!(
                    row["duplicates"],
                    serde_json::json!([{"kind":"semantic_identity"}]),
                    "{row}"
                );
            }
            assert_eq!(canonical.unwrap()["identity"], copied["identity"]);
            assert_eq!(canonical.unwrap()["generation"], copied["generation"]);
            assert_eq!(canonical.unwrap()["family"], copied["family"]);
            assert_eq!(canonical.unwrap()["range"], copied["range"]);
        } else {
            assert_eq!(canonical.unwrap()["outcome"]["posture"], "intact");
            assert_eq!(copied["outcome"]["posture"], "unknown", "{copied}");
            assert_eq!(copied["outcome"]["reason"], "unrecognized_file", "{copied}");
            assert_eq!(copied["family"], "unrecognized");
        }
    } else if target.grammar != super::artifact_inventory::FrameGrammar::Common {
        // Optional checkpoint and pre-retention WAL absence cannot establish
        // missing authority. Assert no fabricated Intact artifact for that path.
        assert!(
            rows.iter().all(|row| row["path"] != path),
            "absent optional journal must not be invented"
        );
    } else {
        let row = canonical.expect("reachable removed artifact must remain a missing observation");
        super::artifact_expectation::require_common_scope(
            row,
            target,
            baseline,
            Some(operator),
            false,
        );
        assert_eq!(row["outcome"]["posture"], "damaged", "{row}");
        assert_eq!(row["outcome"]["cause"], "missing_artifact", "{row}");
    }
}

use std::path::Path;

pub(super) fn observe_copy_media(root: &Path, directory: &Path, stage: &str) {
    let report = super::support::observe_arena_in_separate_process(
        root,
        &directory.join(format!("copy-{stage}.json")),
        "copy-crash",
        stage,
    );
    assert_eq!(report["completeness"], "complete", "{report:#?}");
    let artifacts = report["artifacts"].as_array().unwrap();
    let source = artifacts
        .iter()
        .filter(|artifact| {
            artifact["path"]
                .as_str()
                .unwrap()
                .ends_with("arena-0000000000000001.data")
                && artifact["family"] == "extent_manifest"
                && artifact["identity"]
                    .as_str()
                    .unwrap()
                    .starts_with("arena:0000000000000001:extent:")
                && !artifact["identity"].as_str().unwrap().contains(":chunk:")
        })
        .collect::<Vec<_>>();
    assert!(
        source
            .iter()
            .any(|artifact| artifact["outcome"]["posture"] == "intact"),
        "original source remains independently readable: {source:#?}"
    );
    let destination = artifacts
        .iter()
        .filter(|artifact| {
            artifact["path"]
                .as_str()
                .unwrap()
                .ends_with("arena-0000000000000002.data")
        })
        .collect::<Vec<_>>();
    assert!(!destination.is_empty(), "copied arena must be observed");
    if stage == "killed" {
        assert!(
            destination
                .iter()
                .any(|artifact| artifact["outcome"]["posture"] == "unknown"),
            "unpublished destination is not a selected route: {destination:#?}"
        );
    } else {
        assert!(
            destination
                .iter()
                .any(|artifact| artifact["family"] == "extent_arena_frame"
                    && artifact["identity"]
                        .as_str()
                        .unwrap()
                        .starts_with("arena-accounting:")
                    && artifact["outcome"]["posture"] == "intact"),
            "recovered selected destination has intact arena accounting: {destination:#?}"
        );
    }
}

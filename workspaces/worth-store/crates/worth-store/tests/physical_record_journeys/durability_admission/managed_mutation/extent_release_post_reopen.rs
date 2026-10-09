use std::path::Path;

pub(super) fn reseal_observer_fixture_frame(bytes: &mut [u8]) {
    let checksum = crate::durable_frame_oracle::independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub(super) fn assert_clean_arena_accounting(root: &Path) {
    let directory = tempfile::tempdir().unwrap();
    let report = observe_release_fixture(root, directory.path());
    assert_eq!(report["completeness"], "complete", "{report:#?}");
    let accounting = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| {
            artifact["identity"]
                .as_str()
                .unwrap()
                .starts_with("arena-accounting:")
        })
        .collect::<Vec<_>>();
    assert!(
        !accounting.is_empty(),
        "freshly reopened arena must be accounted offline"
    );
    assert!(
        accounting
            .iter()
            .all(|artifact| artifact["outcome"]["posture"] == "intact"),
        "post-reopen arena accounting must remain exact: {accounting:#?}"
    );
}

pub(super) fn observe_release_fixture(root: &Path, directory: &Path) -> serde_json::Value {
    super::super::independent_arena_observer::observe_arena_in_separate_process(
        root,
        directory,
        "crash-parent",
        "held-extent",
    )
}

use std::path::Path;

pub(super) fn observe_arena(root: &Path, directory: &Path, stage: &str) {
    let report = super::support::observe_arena_in_separate_process(
        root,
        &directory.join(format!("{stage}.json")),
        "append-crash",
        stage,
    );
    assert_eq!(report["completeness"], "complete", "{report:#?}");
    let arenas = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| artifact["path"].as_str().unwrap().contains("/arenas/"))
        .collect::<Vec<_>>();
    assert!(
        !arenas.is_empty(),
        "offline walk must account for arena bytes"
    );
    if stage == "killed" {
        assert_eq!(arenas.len(), 1);
        assert!(
            arenas[0]["outcome"]["posture"] == "unknown"
                && arenas[0]["outcome"]["reason"] == "unrecognized_file",
            "unpublished arena bytes must not be misreported as a routed frame: {arenas:#?}"
        );
    } else {
        assert!(
            arenas
                .iter()
                .all(|artifact| artifact["outcome"]["posture"] == "intact"),
            "{stage} arena observation: {arenas:#?}"
        );
    }
}

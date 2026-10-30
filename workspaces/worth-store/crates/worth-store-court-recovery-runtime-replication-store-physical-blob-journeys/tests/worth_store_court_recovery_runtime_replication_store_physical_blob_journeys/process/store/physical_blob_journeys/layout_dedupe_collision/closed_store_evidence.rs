use std::path::Path;

use super::observe_closed_store_named;

pub(super) fn assert_selected_quarantine_is_intact(root: &Path) {
    let report = observe_closed_store_named(
        root,
        "c11-dedupe-quarantine",
        "forced-unequal-candidate-reopen-rebuild",
    );
    let quarantines = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| artifact["family"] == "blob_dedupe_quarantine")
        .collect::<Vec<_>>();
    assert_eq!(quarantines.len(), 1, "{report}");
    assert_eq!(quarantines[0]["outcome"]["posture"], "intact", "{report}");
}

pub(super) fn assert_reclaimed_orphan_has_no_quarantine(root: &Path) {
    let report = observe_closed_store_named(
        root,
        "c11-dedupe-quarantine-gap",
        "orphan-reclaimed-without-marker",
    );
    assert_eq!(report["completeness"], "complete", "{report}");
    assert!(!report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|artifact| artifact["family"] == "blob_dedupe_quarantine"));
}

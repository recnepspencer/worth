use super::{
    fresh_identity, run_offline_observer, ArtifactInventory, ArtifactRequest, ProcessTreeSnapshot,
};
use serde_json::Value;
use std::path::Path;

mod source_changes;

pub(super) fn require_actual_disagreement(
    observer: &Path,
    reports: &Path,
    clean_request: &ArtifactRequest,
    inventory: &ArtifactInventory,
) {
    let target = inventory
        .granules
        .iter()
        .find(|granule| granule.family == "inline_page")
        .unwrap();
    // The runtime has finished its real bounded inspection and remains alive at
    // the completion/release barrier. Its recorded observation is never edited.
    assert!(clean_request.report.with_extension("complete").is_file());
    assert!(!clean_request.report.with_extension("release").exists());
    let unchanged = ProcessTreeSnapshot::observe_live_diagnostic(&clean_request.root).unwrap();
    let changing = source_changes::SourceChanges::start(&clean_request.root.join(&target.path));
    let report = reports.join("actual-disagreement-offline.json");
    let offline = run_offline_observer(
        observer,
        &clean_request.root,
        &report,
        fresh_identity("actual-disagreement-offline", reports),
        decode_hex(&clean_request.scenario),
    );
    assert!(
        changing.finish() > 1,
        "source metadata changed repeatedly across the bounded walk"
    );
    unchanged.require_unchanged(&clean_request.root).unwrap();
    let row = super::find(&offline.report, target);
    assert_eq!(row["outcome"]["posture"], "indeterminate", "{row}");
    assert_eq!(row["outcome"]["reason"], "source_changed", "{row}");
    let comparison = reports.join("actual-disagreement-comparison.json");
    super::compare(observer, &clean_request.report, &report, &comparison);
    let wire: Value = serde_json::from_slice(&std::fs::read(comparison).unwrap()).unwrap();
    let path = target.path.to_string_lossy().replace('\\', "/");
    let entry = wire["comparisons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path && entry["offset"] == target.offset() as u64)
        .unwrap();
    assert_eq!(entry["agreement"], false);
    assert!(entry["posture_disagreement"].is_object());
    assert_eq!(
        super::find(&wire["runtime"], target)["outcome"]["posture"],
        "intact"
    );
    assert_eq!(
        super::find(&wire["offline"], target)["outcome"]["posture"],
        "indeterminate"
    );
}

fn decode_hex(value: &str) -> [u8; 32] {
    let mut result = [0; 32];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
    }
    result
}

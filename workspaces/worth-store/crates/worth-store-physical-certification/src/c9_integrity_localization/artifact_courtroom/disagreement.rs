use std::path::Path;
use serde_json::Value;
use super::{ArtifactInventory,ArtifactRequest,ArtifactProcessRole,ProcessTreeSnapshot,fresh_identity,run_offline_observer};

pub(super) fn require_actual_disagreement(observer:&Path,executable:&Path,reports:&Path,
    clean_request:&ArtifactRequest,inventory:&ArtifactInventory) {
    let index=inventory.granules.iter().position(|granule|granule.family=="inline_page").unwrap();
    let target=&inventory.granules[index];
    let mut request=clean_request.clone();
    request.role=ArtifactProcessRole::Editor;
    request.poison=Some(index);
    request.operator=super::super::artifact_edit::ArtifactOperator::CoveredByte;
    let before=ProcessTreeSnapshot::observe(&request.root).unwrap();
    let mut editor=super::launch(executable,reports,"actual-disagreement-editor",&request);
    super::wait(&mut editor);
    before.require_exact_one_byte_delta(&request.root,&target.path,
        super::super::artifact_process::covered_byte_offset(target) as u64,1).unwrap();
    let unchanged=ProcessTreeSnapshot::observe(&request.root).unwrap();
    let report=reports.join("actual-disagreement-offline.json");
    // This is a later real byte observation, not an edit of either report. The two
    // source instants intentionally disagree on the same concrete page scope.
    let scenario=decode_hex(&request.scenario);
    let offline=run_offline_observer(observer,&request.root,&report,
        fresh_identity("actual-disagreement-offline",reports),scenario);
    unchanged.require_unchanged(&request.root).unwrap();
    assert_eq!(super::find(&offline.report,target)["outcome"]["posture"],"damaged");
    let comparison=reports.join("actual-disagreement-comparison.json");
    super::compare(observer,&clean_request.report,&report,&comparison);
    let wire:Value=serde_json::from_slice(&std::fs::read(comparison).unwrap()).unwrap();
    let path=target.path.to_string_lossy().replace('\\',"/");
    let entry=wire["comparisons"].as_array().unwrap().iter().find(|entry|
        entry["path"]==path && entry["offset"]==target.offset() as u64).unwrap();
    assert_eq!(entry["agreement"],false);
    assert!(entry["posture_disagreement"].is_object());
    assert_eq!(super::find(&wire["runtime"],target)["outcome"]["posture"],"intact");
    assert_eq!(super::find(&wire["offline"],target)["outcome"]["posture"],"damaged");
}

fn decode_hex(value:&str)->[u8;32] {
    let mut result=[0;32];
    for (index,byte) in result.iter_mut().enumerate() {
        *byte=u8::from_str_radix(&value[index*2..index*2+2],16).unwrap();
    }
    result
}

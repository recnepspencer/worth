mod proposed_families;
use super::*;
use crate::validation::engine::InvariantExecutionMetadata;

fn execution_metadata(ceiling: InvariantCostClass) -> InvariantExecutionMetadata {
    InvariantExecutionMetadata::new(
        InvariantExecutionPoint::SnapshotPublication,
        InvariantObservationKind::Committed,
        VersionId(1),
        VersionId(1),
        InvariantGroupSet::empty(),
        InvariantGroupSet::empty(),
        ceiling,
        InvariantExecutionDisposition::Executed,
        None,
        false,
        None,
        Vec::new(),
        None,
        None,
    )
}

#[test]
fn snapshot_pressure_ceiling_does_not_change_original_validation_metadata() {
    let global = execution_metadata(InvariantCostClass::Global);
    let partition = execution_metadata(InvariantCostClass::Partition);
    assert_ne!(global.max_cost(), partition.max_cost());
    assert_eq!(
        RelationalInvariantMetadataMeaning::from_original(&global),
        RelationalInvariantMetadataMeaning::from_original(&partition),
    );
}

#[test]
fn previous_snapshot_pressure_metadata_refuses_decode() {
    let current = RelationalInvariantMetadataMeaning::from_original(&execution_metadata(
        InvariantCostClass::Global,
    ));
    let mut previous = serde_json::to_value(&current).unwrap();
    previous.as_object_mut().unwrap().insert(
        "declared_cost".into(),
        serde_json::to_value(InvariantCostClass::Partition).unwrap(),
    );
    let bytes = rmp_serde::to_vec_named(&previous).unwrap();
    assert!(rmp_serde::from_slice::<RelationalInvariantMetadataMeaning>(&bytes).is_err());
    let bytes = rmp_serde::to_vec_named(&current).unwrap();
    assert_eq!(
        rmp_serde::from_slice::<RelationalInvariantMetadataMeaning>(&bytes).unwrap(),
        current,
    );
}

use super::*;

#[test]
fn write_receipt_extraction_rejects_source_identity_mismatch() {
    let contract = admitted(
        ProjectionConsumptionSource::from_write_receipt(&write_receipt()),
        binding(&[]),
        ProjectMaterializedFacts::declare().target_identity(),
    )
    .bind_contract()
    .expect("fixture digest must fit");
    let mismatched_receipt = WorthQueryWriteReceipt::test_only(
        phase_four_commit_identity("commit:other"),
        phase_four_snapshot_identity("snapshot:test"),
        WorthQueryMutationTargetClass::Entity,
        Some("tasks"),
        Some(test_entity_identity("task-1")),
        None,
        None,
        None,
    );

    let error = contract
        .extract_from_write_receipt(&mismatched_receipt)
        .unwrap_err();
    assert!(matches!(
        error,
        ProjectionFactExtractionError::SourceIdentityMismatch { .. }
    ));
}

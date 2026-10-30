//! Thirty-two anchors retain identity through a real maintenance publication.
pub(crate) use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::ServingPhysicalRuntime, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest};
pub(crate) fn rewrite(serving: &ServingPhysicalRuntime, placement: AdmittedRecordPlacementPolicy) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x52; 32]),
        )
        .unwrap();
    let result = submission
        .rewrite_selected_inline_segment(
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw();
    let worth_proof::TransitionOutcome::Success(
        worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
    ) = result
    else {
        panic!("maintenance must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
}

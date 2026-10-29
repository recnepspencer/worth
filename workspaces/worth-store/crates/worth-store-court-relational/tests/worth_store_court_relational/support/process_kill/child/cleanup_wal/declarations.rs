//! A bounded real WAL prefix for the recovery cleanup interruption fixture.
pub(crate) use {std::num::NonZeroU64};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch};
pub(crate) use {worth_store_relational::RelationalStoreHome};

pub(crate) fn cover_segments_at_next_checkpoint(home: &RelationalStoreHome, limit: NonZeroU64) {
    // All four anchors belong to the home. Append unanchored records through
    // Store's ordinary prepared-mutation execution instead. Sixteen
    // 64 KiB records at a 512 KiB limit give 1 MiB of owner payload and more
    // than two segments of real redo without changing any Relational anchor.
    let payload_bytes = usize::try_from(limit.get() / 8).unwrap();
    let submission = home
        .certification_serving()
        .certification_record_submission();
    for generation in 0_u8..16 {
        let key = submission
            .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([generation; 32]))
            .unwrap();
        let prepared = submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([vec![generation; payload_bytes]]).unwrap(),
                crate::open_refusals::home_placement(),
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                    key,
                    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw();
        let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
            prepared
        else {
            panic!("the cleanup fixture's distinct append prepares");
        };
        assert!(matches!(
            prepared.execute(),
            worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
        ));
    }
    eprintln!(
        "cleanup fixture: WAL limit={} bytes; owner payload={} bytes",
        limit,
        payload_bytes * 16
    );
}
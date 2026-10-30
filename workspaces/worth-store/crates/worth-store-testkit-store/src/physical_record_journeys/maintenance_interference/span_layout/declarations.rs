pub use std::path::Path;

pub use worth_proof::TransitionOutcome;
pub use worth_store_physical_backend::MediaOperationRole;
pub use {
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationDenial,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::RecordAppendDenial,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub use crate::physical_record_journeys::maintenance_interference::segment_files;
pub use {
    crate::physical_record_journeys::maintenance_interference::rewrite_scope::open_wide,
    crate::physical_record_journeys::maintenance_interference::rewrite_scope::publish_span,
    crate::physical_record_journeys::maintenance_interference::rewrite_scope::reopen_wide,
    crate::physical_record_journeys::maintenance_interference::rewrite_scope::rewrite,
    crate::physical_record_journeys::maintenance_interference::rewrite_scope::wide_placement,
    crate::physical_record_journeys::maintenance_interference::rewrite_scope::RECORD_BYTES,
};

pub const PAGE_BYTES: u64 = 16 * 1024;

pub fn read_all(
    serving: &ServingPhysicalRuntime,
    ids: &[worth_store::physical_runtime::PhysicalRecordId],
) {
    let limits = worth_store::physical_runtime::RecordReadLimits::new(
        worth_store::physical_runtime::RecordByteLimit::new(RECORD_BYTES as u32).unwrap(),
    );
    let reader = serving.records().unwrap();
    for (ordinal, id) in ids.iter().copied().enumerate() {
        let mut session = reader.open(id, limits).unwrap();
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(chunk.bytes(), vec![ordinal as u8; RECORD_BYTES].as_slice());
    }
}

pub fn denied_rewrite(
    serving: &ServingPhysicalRuntime,
    pages: u32,
) -> PhysicalMutationPreparationDenial {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x7D; 32]),
        )
        .unwrap();
    match submission
        .rewrite_selected_inline_pages(
            wide_placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
            pages,
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        worth_proof::TransitionOutcome::Success(_) => panic!("{pages} pages were admitted"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("{pages} pages were deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("{pages} pages were stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("{pages} pages required rebind")
        }
        worth_proof::TransitionOutcome::Failed(_) => panic!("{pages} pages failed closed"),
    }
}

pub fn file_len(root: &Path, name: &str) -> u64 {
    std::fs::metadata(root.join("families/records/segments").join(name))
        .unwrap()
        .len()
}

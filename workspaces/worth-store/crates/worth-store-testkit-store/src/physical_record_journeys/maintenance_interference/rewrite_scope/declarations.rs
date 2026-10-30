pub use worth_proof::TransitionOutcome;
pub use worth_store::physical_runtime::PhysicalWalPolicy;
pub use worth_store_physical_backend::MediaOperationRole;
pub use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationDenial,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalReadProtectionPolicy,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordAppendDenial,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::SegmentPageCount,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};

pub const PAGE_BYTES: u64 = 16 * 1024;
pub const RECORD_BYTES: usize = 7_500;

pub fn open_wide(root: &std::path::Path) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let media = crate::physical_record_journeys::media(root);
    let durability = crate::physical_record_journeys::durability_with_wal_policy(
        &media,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(
                std::num::NonZeroU64::new(512 * 1024).unwrap(),
            ),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                std::num::NonZeroU32::new(64).unwrap(),
            ),
        ),
    );
    crate::physical_record_journeys::success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format,
                wide_placement(),
                access,
                durability,
            )
            .with_residency_policy(
                crate::physical_record_journeys::maintenance_interference::rewrite_axis_residency(
                    format,
                    1024 * 1024,
                ),
            )
            .with_read_protection_policy(
                worth_store::physical_runtime::PhysicalReadProtectionPolicy::default(),
            ),
        ),
    )
}

pub fn reopen_wide(
    root: &std::path::Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let media = crate::physical_record_journeys::media(root);
    let durability = crate::physical_record_journeys::durability_with_wal_policy(
        &media,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(
                std::num::NonZeroU64::new(512 * 1024).unwrap(),
            ),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                std::num::NonZeroU32::new(64).unwrap(),
            ),
        ),
    );
    crate::physical_record_journeys::success(
        media.open_record_store(
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
                .with_residency_policy(crate::physical_record_journeys::maintenance_interference::rewrite_axis_residency(format, 1024 * 1024))
                .with_read_protection_policy(worth_store::physical_runtime::PhysicalReadProtectionPolicy::default()),
        ),
    )
}

pub fn wide_placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(4).unwrap())
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(16).unwrap())
        .admit(format)
        .unwrap()
}

pub fn publish_span(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    pages: u32,
) -> Vec<PhysicalRecordId> {
    let records: Vec<Vec<u8>> = (0..pages)
        .map(|ordinal| vec![ordinal as u8; RECORD_BYTES])
        .collect();
    let mut material = [0x6A; 32];
    material[..8].copy_from_slice(&u64::from(pages).to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter(
                records.iter().map(Vec::as_slice),
            )
            .unwrap(),
            *policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("{pages} records did not stay prepared")
        }
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("{pages} records denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            panic!("{pages} records deferred: {deferred:?}")
        }
        worth_proof::TransitionOutcome::Stale(stale) => panic!("{pages} records stale: {stale:?}"),
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!("{pages} records rebind: {rebind:?}")
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("{pages} records failed: {failure:?}")
        }
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(done) => {
            done.into_acknowledgment().record_ids().collect()
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("{pages} records had no effect: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("{pages} records became indeterminate at {:?}", fate.stage())
        }
    }
}

pub fn publish(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    mark: u8,
) -> PhysicalRecordId {
    let mut bytes = vec![mark; RECORD_BYTES];
    bytes[0] = mark;
    crate::physical_record_journeys::maintenance_interference::append(
        serving, *policy, ordinal, &bytes,
    )
}

pub fn rewrite(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    pages: u32,
) {
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_pages(
            *policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
            pages,
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Success(_) => panic!("{pages} pages did not stay prepared"),
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("{pages} pages denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            panic!("{pages} pages deferred: {deferred:?}")
        }
        worth_proof::TransitionOutcome::Stale(stale) => panic!("{pages} pages stale: {stale:?}"),
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!("{pages} pages rebind: {rebind:?}")
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("{pages} pages failed: {failure:?}")
        }
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("{pages} pages had no effect: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("{pages} pages became indeterminate at {:?}", fate.stage())
        }
    }
}

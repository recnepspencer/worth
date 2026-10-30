pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRetirementDenial,
    worth_store::physical_runtime::RecordReadDenial,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::append,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::append_copies,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_files,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_ids,
};

pub(crate) fn retirement_examination(roots: u32) -> u64 {
    use std::num::NonZeroU32;
    use worth_store::physical_runtime::PhysicalReadProtectionPolicy;
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize_protection(
        &parent.path().join("store"),
        PhysicalReadProtectionPolicy::new(
            NonZeroU32::new(64).unwrap(),
            NonZeroU32::new(roots).unwrap(),
        ),
    );
    let policy = placement();
    let mut readers = Vec::new();
    // Segment capacity is 8 pages. Stop on the first page of a segment so the
    // rewrite displaces that generation, and keep a reader on that root.
    let total = 8 * (roots - 1) + 1;
    for index in 0..total {
        append(&serving, policy, u64::from(index) + 1, b"root");
        if readers.len() as u32 == roots {
            readers.remove(0);
        }
        readers.push(serving.records().unwrap());
    }
    rewrite_current(&serving, policy, 20_000 + u64::from(roots));
    let examined = serving
        .read_protection_observer()
        .snapshot()
        .examined_entries();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
    );
    let delta = serving
        .read_protection_observer()
        .snapshot()
        .examined_entries()
        - examined;
    assert!(
        delta >= 1,
        "retirement returned Protected without consulting the index"
    );
    drop(readers);
    serving.close();
    delta
}

pub(crate) fn rewrite_current(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
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
        .rewrite_selected_inline_segment(
            policy,
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
            panic!("rewrite {ordinal} did not stay prepared")
        }
        worth_proof::TransitionOutcome::Denied(_) => panic!("rewrite {ordinal} denied"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("rewrite {ordinal} deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("rewrite {ordinal} stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => panic!("rewrite {ordinal} rebind"),
        worth_proof::TransitionOutcome::Failed(_) => panic!("rewrite {ordinal} failed"),
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(_) => {
            panic!("rewrite {ordinal} had no effect")
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("rewrite {ordinal} indeterminate")
        }
    }
}

pub(crate) fn wal_rotations(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) -> u64 {
    serving
        .record_submission()
        .wal_observation()
        .map(|observation| observation.rotations())
        .unwrap_or(0)
}

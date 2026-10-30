pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalReadProtectionPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::RecordReadDenial,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize_with_wal,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement,
};

#[derive(Debug)]
pub(crate) enum Attempt {
    Published(PhysicalRecordId),
    NoEffect(PhysicalMutationProvenNoEffectCause),
}

pub(crate) fn expect_published(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    bytes: &[u8],
) -> PhysicalRecordId {
    match attempt(serving, policy, ordinal, bytes) {
        crate::physical_record_journeys::maintenance_interference::retention::Attempt::Published(id) => id,
        crate::physical_record_journeys::maintenance_interference::retention::Attempt::NoEffect(cause) => panic!("append {ordinal} had no effect: {cause:?}"),
    }
}

pub(crate) fn attempt(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    bytes: &[u8],
) -> Attempt {
    let mut material = [0x6A; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([bytes]).unwrap(),
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
            panic!("append {ordinal} did not stay prepared")
        }
        worth_proof::TransitionOutcome::Denied(_) => panic!("append {ordinal} denied"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("append {ordinal} deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("append {ordinal} stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => panic!("append {ordinal} rebind"),
        worth_proof::TransitionOutcome::Failed(_) => panic!("append {ordinal} failed"),
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(done) => {
            crate::physical_record_journeys::maintenance_interference::retention::Attempt::Published(
                done.into_acknowledgment().record_ids().next().unwrap(),
            )
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            crate::physical_record_journeys::maintenance_interference::retention::Attempt::NoEffect(
                fate.cause(),
            )
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!(
                "append {ordinal} became indeterminate at {:?}",
                fate.stage()
            )
        }
    }
}

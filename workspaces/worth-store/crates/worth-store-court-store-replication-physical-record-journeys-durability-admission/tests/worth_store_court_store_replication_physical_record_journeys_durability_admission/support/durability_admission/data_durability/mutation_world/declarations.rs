pub(crate) use std::path::Path;

pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use worth_store::physical_runtime::certification::MediaFaultDirective;
pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::SealedPhysicalDurabilityGroupMembers,
};
pub(crate) use {
    worth_store_physical_backend::CertificationMediaFaultActivation,
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaOperationRole,
};

pub(crate) fn append(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
    batch: RecordAppendBatch,
) -> SealedPhysicalDurabilityGroupMembers {
    let key = submission.issue_idempotency_key(material).unwrap();
    let prepared = match submission
        .prepare_durable_append(
            batch,
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(
                    worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("the real durable preparation path must succeed"),
    };
    match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new())) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => {
            appended
        }
        _ => panic!("the real WAL append path must succeed"),
    }
}

pub(crate) fn synchronize(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    appended: SealedPhysicalDurabilityGroupMembers,
) -> worth_store::physical_runtime::WalDurablePhysicalMutation {
    match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(durable) => durable
            .into_members()
            .into_vec()
            .pop()
            .expect("a singleton group derives one durable member"),
        _ => panic!("the real WAL barrier must mint durable authority"),
    }
}

pub(crate) fn fault_scheduled_media(
    root: &Path,
    target: u64,
    directive: MediaFaultDirective,
) -> (
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    CertificationMediaFaultActivation,
) {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedWrite,
                target,
                directive,
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("fault-scheduled media admission must succeed"),
    };
    (media, activation)
}

pub(crate) fn fault_scheduled_media_at_identified_ordinal(
    root: &Path,
    target: u64,
    directive: MediaFaultDirective,
) -> worth_store::physical_runtime::MediaOwnedPhysicalRuntime {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedWrite,
                target,
                directive,
            )
            .for_identified_operation_ordinal()])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("fault-scheduled media admission must succeed"),
    }
}

pub(crate) fn certification_media(
    root: &Path,
) -> worth_store::physical_runtime::MediaOwnedPhysicalRuntime {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    match runtime.try_admit_filesystem_media(admission).into_raw() {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("certification media admission must succeed"),
    }
}

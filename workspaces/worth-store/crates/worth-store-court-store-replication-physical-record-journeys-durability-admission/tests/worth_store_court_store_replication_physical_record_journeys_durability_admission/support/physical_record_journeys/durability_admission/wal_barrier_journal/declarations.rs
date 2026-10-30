pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store::physical_runtime::certification::MediaFaultDirective;
pub(crate) use {
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalEffectRecoveryObligation,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::PhysicalWorkRecoveryDisposition,
    worth_store::physical_runtime::PhysicalWorkRecoveryTarget,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_backend::CertificationMediaFaultActivation,
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaCounterSnapshot,
    worth_store_physical_backend::MediaOperationRole,
};

pub(crate) use crate::physical_record_journeys::durability_admission::wal_barrier::append;
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_with_group_limit,
};

pub(crate) fn commit(
    serving: &ServingPhysicalRuntime,
    material: u8,
) -> PhysicalWalGroupBarrierOutcome {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([material; 32]),
    );
    submission.synchronize_appended_wal_group(appended)
}

pub(crate) fn indeterminate_recovery(
    outcome: PhysicalWalGroupBarrierOutcome,
) -> PhysicalEffectRecoveryObligation {
    match outcome {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Indeterminate(barrier) => {
            barrier.recovery_obligation()
        }
        _ => panic!("a failed WAL flush must leave the barrier indeterminate"),
    }
}

pub(crate) fn barrier_after_arming(
    serving: &ServingPhysicalRuntime,
    activations: &[CertificationMediaFaultActivation],
    material: u8,
) -> PhysicalWalGroupBarrierOutcome {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([material; 32]),
    );
    for activation in activations {
        activation.arm().unwrap();
    }
    submission.synchronize_appended_wal_group(appended)
}

pub(crate) fn serving_with_faults(
    root: &Path,
    faults: &[(MediaOperationRole, MediaFaultDirective)],
) -> (
    ServingPhysicalRuntime,
    Vec<CertificationMediaFaultActivation>,
) {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let activations = faults
        .iter()
        .map(|_| authority.one_shot_activation())
        .collect::<Vec<_>>();
    let rules = faults
        .iter()
        .zip(&activations)
        .map(|((role, directive), activation)| {
            authority
                .rule(*role, 1, directive.clone())
                .for_nth_identified_operation_after_activation(
                    activation.clone(),
                    std::num::NonZeroU64::MIN,
                )
        })
        .collect();
    let schedule = authority.schedule(rules).unwrap();
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
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, _, access) = configuration();
    let serving =
        worth_store_testkit_store::physical_record_journeys::success(media.open_record_store(
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, policy),
        ));
    (serving, activations)
}

pub(crate) fn attempts(counters: MediaCounterSnapshot, role: MediaOperationRole) -> u64 {
    counters.attempts_for(role)
}

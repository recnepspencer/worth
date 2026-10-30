pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use worth_store::physical_runtime::certification::MediaFaultDirective;
pub(crate) use {
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalMutationAdmissionDisposition,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::PhysicalWalReclamationObservation,
    worth_store::physical_runtime::PreparedPhysicalMutation,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_backend::CertificationMediaFaultActivation,
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaOperationRole, worth_store_physical_backend::MediaPauseGate,
};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy, worth_store_testkit_store::physical_record_journeys::success};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_retained_wal_tail::durable_group, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_retained_wal_tail::publish_after_foreground_rotations, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_retained_wal_tail::rotation_world};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::wal_rotation::append_group, worth_store_testkit_store::physical_record_journeys::durability_admission::wal_rotation::wal_policy};

pub(crate) fn assert_unmaterialized_candidates(
    root: &std::path::Path,
    serving: &ServingPhysicalRuntime,
) {
    assert!(std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation(),
        1
    );
}

pub(crate) fn prepare_with_key(
    submission: &worth_store::physical_runtime::PhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    key: PhysicalMutationIdempotencyKey,
    payload: &[u8],
) -> PreparedPhysicalMutation {
    let request = worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
        ),
    );
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
            placement,
            request,
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("exact-key reclamation replay was not admitted"),
    }
}

pub(crate) fn fault_rotation_world(
    store_root: &std::path::Path,
    directive: MediaFaultDirective,
) -> (
    ServingPhysicalRuntime,
    AdmittedRecordPlacementPolicy,
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
                worth_store_physical_backend::MediaOperationRole::Delete,
                1,
                directive,
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let (serving, placement) =
        admit_fault_rotation_world(store_root, admission.with_fault_schedule(schedule));
    (serving, placement, activation)
}

pub(crate) struct SecondDeleteFaultRotationWorld {
    pub(crate) serving: ServingPhysicalRuntime,
    pub(crate) placement: AdmittedRecordPlacementPolicy,
    pub(crate) first_delete_gate: MediaPauseGate,
    pub(crate) pause_activation: CertificationMediaFaultActivation,
    pub(crate) fail_activation: CertificationMediaFaultActivation,
}

pub(crate) fn second_delete_fault_rotation_world(
    store_root: &std::path::Path,
    directive: MediaFaultDirective,
) -> SecondDeleteFaultRotationWorld {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let first_delete_gate = authority.pause_gate();
    let pause_activation = authority.one_shot_activation();
    let fail_activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![
            authority
                .rule(
                    worth_store_physical_backend::MediaOperationRole::Delete,
                    1,
                    directive,
                )
                .for_next_identified_operation_after_activation(fail_activation.clone()),
            authority
                .rule(
                    worth_store_physical_backend::MediaOperationRole::Delete,
                    1,
                    worth_store::physical_runtime::certification::MediaFaultDirective::PauseBefore(
                        first_delete_gate.clone(),
                    ),
                )
                .for_next_identified_operation_after_activation(pause_activation.clone()),
        ])
        .unwrap();
    let (serving, placement) =
        admit_fault_rotation_world(store_root, admission.with_fault_schedule(schedule));
    SecondDeleteFaultRotationWorld {
        serving,
        placement,
        first_delete_gate,
        pause_activation,
        fail_activation,
    }
}

pub(crate) fn admit_fault_rotation_world(
    store_root: &std::path::Path,
    admission: FilesystemMediaAdmission,
) -> (ServingPhysicalRuntime, AdmittedRecordPlacementPolicy) {
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(store_root).unwrap(),
    )
    .unwrap();
    let media_owner = match runtime.try_admit_filesystem_media(admission).into_raw() {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("fault-scheduled WAL reclamation media was not admitted"),
    };
    let durability = durability_with_wal_policy(&media_owner, wal_policy(6));
    let (format, placement, access) = configuration();
    let serving = success(media_owner.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ),
    ));
    (serving, placement)
}

pub(crate) fn assert_reclaimed_prefix(
    observation: &PhysicalWalReclamationObservation,
    expected_segments: u32,
) {
    match observation {
        worth_store::physical_runtime::PhysicalWalReclamationObservation::Reclaimed(report) => {
            assert_eq!(report.planned_segments(), expected_segments);
            assert_eq!(report.reclaimed_segments(), expected_segments);
            assert!(report.reclaimed_bytes() > 0);
            assert_eq!(report.first_unreclaimed(), None);
        }
        other => panic!("checkpoint did not reclaim its obsolete WAL prefix: {other:?}"),
    }
}

pub use std::path::Path;
pub use {std::time::Duration, std::time::Instant};

pub use worth_signal::facade::TemporalDuration;
pub use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub use {
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint,
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionPauseGate,
    worth_store::physical_runtime::certification::MediaFaultDirective,
};
pub use {
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointHandle,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointProgressPhase,
    worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalCheckpointStartDeferred,
    worth_store::physical_runtime::PhysicalCheckpointStartStale,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::PhysicalStoreCloseOutcome,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub use {
    worth_store_physical_backend::CertificationMediaFaultActivation,
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaOperationRole,
};

pub use {
    crate::physical_record_journeys::configuration, crate::physical_record_journeys::durability,
    crate::physical_record_journeys::serving_from_initialization,
    crate::physical_record_journeys::success,
};

pub fn pause_checkpoint_at_phase(
    gate: &CertificationPhysicalExecutionPauseGate,
    handle: &PhysicalCheckpointHandle,
    target: PhysicalCheckpointProgressPhase,
) -> usize {
    let mut arrival = 0;
    loop {
        assert!(gate.await_arrivals(arrival + 1));
        if handle.progress().phase() == target {
            return arrival;
        }
        assert!(arrival < 64, "checkpoint did not reach {target:?}");
        let release = gate.release_arrival(arrival).unwrap();
        assert!(release.await_resumption());
        arrival += 1;
    }
}

pub fn serving_with_durable_wal(root: &Path, material: u8) -> ServingPhysicalRuntime {
    let serving = serving_from_initialization(root);
    append_durable_wal(&serving, material);
    serving
}

pub fn append_durable_wal(serving: &ServingPhysicalRuntime, material: u8) {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let prepared = crate::physical_record_journeys::durability_admission::wal_append::prepared(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([material; 32]),
        b"checkpoint-lifecycle-redo",
    );
    let appended = match submission
        .append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new()))
    {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => {
            appended
        }
        _ => panic!("checkpoint lifecycle setup requires an appended WAL member"),
    };
    assert!(matches!(
        submission.synchronize_appended_wal_group(appended),
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
}

pub fn fault_scheduled_serving(
    root: &Path,
) -> (ServingPhysicalRuntime, CertificationMediaFaultActivation) {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedWrite,
                1,
                worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
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
        _ => panic!("fault-scheduled checkpoint media admission must succeed"),
    };
    let (format, placement, access) = configuration();
    let policy = durability(&media);
    let serving = success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    (serving, activation)
}

pub fn checkpoint_request(key: u8) -> PhysicalCheckpointRequest {
    worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000)
                .expect("deadline is positive"),
        ),
    )
}

pub fn start(
    serving: &ServingPhysicalRuntime,
    request: PhysicalCheckpointRequest,
) -> worth_store::physical_runtime::PhysicalCheckpointHandle {
    match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint lifecycle setup requires an admitted handle"),
    }
}

pub fn await_checkpoint_admission_stop(
    submission: &worth_store::physical_runtime::PhysicalCheckpointSubmission,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if matches!(
            submission.start(checkpoint_request(32)).into_raw(),
            worth_proof::TransitionOutcome::Stale(
                worth_store::physical_runtime::PhysicalCheckpointStartStale::RuntimeClosing
            )
        ) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "checkpoint close did not stop admission"
        );
        std::thread::yield_now();
    }
}

pub fn candidate_path(root: &Path, sequence: u64) -> std::path::PathBuf {
    root.join(format!("staging/checkpoint-{sequence:016x}.candidate"))
}

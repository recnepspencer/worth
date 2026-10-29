//! A genuine one-shot C8/Store seal must freshly read WAL before Serving.
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalResidencyDimension as Dimension, std::num::NonZeroU64, std::path::Path, worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial, worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope, worth_proof::TransitionOutcome, crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census, crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::checkpoint, std::fs, crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::open_with_policy, crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::recover_core, worth_store_testkit_recovery_runtime::production_entry::release_reopen, crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family};
pub(crate) use {worth_store::physical_runtime::certification::MediaOperationRole};
pub(crate) use {worth_store::physical_runtime::PhysicalRecoveryObservationAllocationDenial, worth_store::physical_runtime::RecordBootstrapDenial, worth_store::physical_runtime::RecoveryWalArtifactView, worth_store::physical_runtime::RecoveryWalReadFailureView};

pub(crate) fn deny_changed_wal_shape(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    policy: AdmittedPhysicalRecordResidencyPolicy,
) {
    for extra_file in [false, true] {
        let before = snapshot_family(root);
        let core = recover_core(root);
        let observer = core.certification_residency_allocations();
        let seal = core
            .certification_release_media_into_checkpoint_custody()
            .expect("fresh genuine seal before external mutation");
        let (name, _) = wal_payload_sizes(root).into_iter().next().unwrap();
        let path = root.join("families/wal").join(name);
        let original = std::fs::read(&path).unwrap();
        let extra = root.join("families/wal/unrecognized-tail.wal");
        if extra_file {
            assert!(!extra.exists());
            std::fs::write(&extra, b"unknown actual directory entry").unwrap();
        } else {
            let mut enlarged = original.clone();
            enlarged.push(0x57);
            std::fs::write(&path, enlarged).unwrap();
        }
        let altered = snapshot_family(root);
        let worth_proof::TransitionOutcome::Denied(denial) =
            open_with_policy(root, seal, format, policy).into_raw()
        else {
            panic!("actual post-seal WAL shape change must not become Serving");
        };
        assert_eq!(
            denial.reason(),
            worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch
        );
        let runtime = denial.into_runtime();
        assert_eq!(runtime.media_counters().replacements(), 0);
        assert_eq!(runtime.media_counters().deletions(), 0);
        assert_eq!(
            runtime
                .media_counters()
                .completed_operations_for(worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite),
            0
        );
        assert_eq!(
            runtime
                .media_counters()
                .completed_operations_for(worth_store::physical_runtime::certification::MediaOperationRole::Append),
            0
        );
        assert_eq!(
            snapshot_family(root),
            altered,
            "only the deliberate external mutation exists"
        );
        runtime.close();
        crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census::assert_disposed("post-seal shape mismatch disposal", |dimension| {
            let counters = observer.snapshot().for_dimension(dimension);
            (
                counters.active_units(),
                counters.admitted_units(),
                counters.released_units(),
            )
        });
        if extra_file {
            std::fs::remove_file(&extra).unwrap();
        } else {
            std::fs::write(&path, &original).unwrap();
        }
        assert_eq!(snapshot_family(root), before);
    }
}

pub(crate) fn wal_payload_sizes(root: &Path) -> Vec<(std::ffi::OsString, u64)> {
    let mut payloads = Vec::new();
    for entry in std::fs::read_dir(root.join("families/wal")).expect("actual retained WAL directory") {
        let entry = entry.unwrap();
        assert!(
            entry.file_type().unwrap().is_file(),
            "genuine WAL fixture has regular segment files"
        );
        payloads.push((entry.file_name(), entry.metadata().unwrap().len()));
    }
    assert!(!payloads.is_empty());
    payloads
}
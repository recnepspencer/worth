//! Selector and addressed-root funding failures use genuine released, checkpointed Store media.

use super::*;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    PhysicalOperationAllocationScope as Scope, PhysicalRecordResidencyPolicy,
    PhysicalResidencyDimension as Dimension, PhysicalSpeculativeWorkKind as Kind,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, DurableRootSelector, RecordArtifactFile, ROOT_SELECTOR_BYTES,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryRootProtocolArtifact as Artifact, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};

#[test]
fn selector_and_root_reads_require_original_and_native_capacity_before_allocation() {
    std::thread::Builder::new()
        .name("release-source-read-funding".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            let store = world.serving().store_identity();
            let request = PhysicalCheckpointRequest::fuzzy(
                PhysicalCheckpointIdempotencyKey::new([0xd5; 32]),
                PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
            );
            let TransitionOutcome::Success(handle) =
                world.serving().checkpoints().start(request).into_raw()
            else {
                panic!("real released world must admit its covering checkpoint");
            };
            let PhysicalCheckpointOutcome::Completed(checkpoint) = handle.wait() else {
                panic!("real released world must publish its covering checkpoint");
            };
            let checkpoint_identity = checkpoint.basis().identity();
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let before = snapshot_family(root);

            let configurations = [
                (
                    ROOT_SELECTOR_BYTES as u64 - 1,
                    PhysicalRecoveryStaticConfiguration::current(),
                ),
                (
                    24 << 20,
                    PhysicalRecoveryStaticConfiguration::current()
                        .with_residency_policy(tiny_recovery_policy())
                        .expect("tiny Recovery policy retains the actual configured format"),
                ),
            ];
            for (index, (memory, configuration)) in configurations.into_iter().enumerate() {
                let outcome = WorthStoreRecovery::recover(
                    super::super::recovery_request::request_with_configuration(
                        root,
                        memory,
                        configuration,
                    ),
                );
                let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
                    panic!("first selector must deny at its funded allocation: {outcome:?}");
                };
                assert_eq!(blocked.store_identity(), store);
                assert_eq!(blocked.recovery_effects(), 0);
                let evidence = blocked.evidence();
                assert_eq!(evidence.counters.bytes_observed, 0);
                assert_eq!(evidence.counters.selector_slots, 0);
                assert_eq!(evidence.counters.current_selector_integrity_admissions, 0);
                assert_eq!(evidence.counters.current_selector_interpretations, 0);
                assert_eq!(evidence.counters.current_root_integrity_admissions, 0);
                let [PhysicalRecoverySourceDenial::SourceReadAllocation {
                    artifact: Artifact::CurrentSelector,
                    boundary: Boundary::ReadBuffer,
                    requested,
                    cause: Cause::Residency(cause),
                }] = evidence.source_denials.as_slice()
                else {
                    panic!(
                        "first-selector source allocation must preserve its exact cause: {:?}",
                        evidence.source_denials
                    );
                };
                assert_eq!(*requested, ROOT_SELECTOR_BYTES as u64);
                match (index, cause) {
                    (
                        0,
                        PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
                    ) => {
                        assert_eq!(*required, ROOT_SELECTOR_BYTES as u64);
                        assert_eq!(*admitted, memory);
                    }
                    (1, PhysicalRecoveryRejoinResidentDenial::OperationAllocation(allocation)) => {
                        let pressure = allocation
                            .pressure()
                            .expect("actual native Recovery scope pressure");
                        assert_eq!(pressure.basis().store_identity(), store);
                        assert_eq!(pressure.scope(), Scope::Recovery);
                        assert_eq!(
                            pressure.dimension(),
                            Dimension::OperationScope(Scope::Recovery)
                        );
                        assert_eq!(pressure.requested(), ROOT_SELECTOR_BYTES as u64);
                        assert_eq!(pressure.limit(), 1);
                        assert!(!pressure.effect_may_have_started());
                    }
                    other => panic!("wrong first-read funding cause: {other:?}"),
                }
                assert_eq!(snapshot_family(root), before);
            }

            let records = root.join("families/records");
            let selector = DurableRootSelector::decode(
                &fs::read(records.join(RecordArtifactFile::CurrentRootSelector.file_name()))
                    .expect("actual current selector remains readable after funding denials"),
            )
            .expect("actual current selector identifies its addressed root");
            let generation = selector.root_generation();
            let root_bytes = fs::metadata(
                records
                    .join("roots")
                    .join(RecordArtifactFile::RootManifest { generation }.file_name()),
            )
            .expect("actual addressed root remains present")
            .len();
            let selector_bytes = ROOT_SELECTOR_BYTES as u64;
            let scratch_bytes =
                DurablePhysicalRootManifest::maximum_encoding_scratch_bytes() as u64;
            for (boundary, requested, required, observed) in [
                (
                    Boundary::ReadBuffer,
                    root_bytes,
                    selector_bytes + root_bytes,
                    selector_bytes,
                ),
                (
                    Boundary::CanonicalValidation,
                    scratch_bytes,
                    selector_bytes + root_bytes + scratch_bytes,
                    selector_bytes + root_bytes,
                ),
            ] {
                let memory = required - 1;
                let outcome = WorthStoreRecovery::recover(
                    super::super::recovery_request::request_with_configuration(
                        root,
                        memory,
                        PhysicalRecoveryStaticConfiguration::current(),
                    ),
                );
                let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
                    panic!(
                        "addressed root must deny before its unfunded {boundary:?}: {outcome:?}"
                    );
                };
                assert_eq!(blocked.store_identity(), store);
                assert_eq!(blocked.recovery_effects(), 0);
                let evidence = blocked.evidence();
                assert_eq!(evidence.counters.bytes_observed, observed);
                assert_eq!(evidence.counters.current_selector_integrity_admissions, 1);
                assert_eq!(evidence.counters.current_selector_interpretations, 1);
                assert_eq!(evidence.counters.current_root_integrity_admissions, 0);
                let [PhysicalRecoverySourceDenial::SourceReadAllocation {
                    artifact:
                        Artifact::CurrentRoot {
                            generation: denied_generation,
                        },
                    boundary: denied_boundary,
                    requested: denied_requested,
                    cause:
                        Cause::Residency(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                            required: denied_required,
                            admitted,
                        }),
                }] = evidence.source_denials.as_slice()
                else {
                    panic!(
                        "addressed-root source allocation must preserve its exact cause: {:?}",
                        evidence.source_denials
                    );
                };
                assert_eq!(*denied_generation, generation);
                assert_eq!(*denied_boundary, boundary);
                assert_eq!(*denied_requested, requested);
                assert_eq!(*denied_required, required);
                assert_eq!(*admitted, memory);
                assert_eq!(snapshot_family(root), before);
            }

            let outcome = WorthStoreRecovery::recover(super::super::request(root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("healthy original/native capacity must reopen the same media: {outcome:?}");
            };
            assert_eq!(handoff.core().store_identity(), store);
            assert_eq!(handoff.core().recovery_effect_count(), 0);
            assert_eq!(
                handoff
                    .selected_sources()
                    .checkpoint()
                    .unwrap()
                    .checkpoint()
                    .source()
                    .identity(),
                checkpoint_identity
            );
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("independent Store rejoin seals real selected custody");
            let serving = super::super::admit_serving_with_seal(root, seal);
            assert_eq!(serving.store_identity(), store);
            assert_eq!(snapshot_family(root), before);
            assert!(!serving.close().residency().requires_inspection());
        })
        .expect("source read funding worker")
        .join()
        .expect("source read funding worker did not panic");
}

fn tiny_recovery_policy() -> AdmittedPhysicalRecordResidencyPolicy {
    recovery_policy_with_scope(1)
}

pub(super) fn recovery_policy_with_scope(
    usable_recovery: u64,
) -> AdmittedPhysicalRecordResidencyPolicy {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let canonical = AdmittedPhysicalRecordResidencyPolicy::canonical(format);
    let bytes = |value| NonZeroU64::new(value).unwrap();
    let count = |value| std::num::NonZeroU32::new(value).unwrap();
    const PROGRESS_HEADROOM: u64 = 64 << 10;
    let mut builder = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(canonical.total_bytes()))
        .resident_bytes(bytes(canonical.resident_bytes()))
        .metadata_bytes(bytes(canonical.metadata_bytes()))
        .frame_entries(count(canonical.frame_entries()))
        .pinned_frames(count(canonical.pinned_frames()))
        .pin_leases(count(canonical.pin_leases()))
        .dirty_frames(count(canonical.dirty_frames()))
        .dirty_replacement_bytes(bytes(canonical.dirty_replacement_bytes()))
        .operation_bytes(bytes(canonical.operation_bytes()))
        .progress_headroom_bytes(PROGRESS_HEADROOM);
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        let maximum = if scope == Scope::Recovery {
            PROGRESS_HEADROOM + usable_recovery
        } else {
            canonical.scope_bytes(scope)
        };
        builder = builder.scope_bytes(scope, bytes(maximum));
    }
    for kind in [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind] {
        builder = builder.speculative_frames(kind, count(canonical.speculative_frames(kind)));
    }
    builder
        .admit(format)
        .into_result()
        .expect("actual policy with bounded usable Recovery bytes")
}

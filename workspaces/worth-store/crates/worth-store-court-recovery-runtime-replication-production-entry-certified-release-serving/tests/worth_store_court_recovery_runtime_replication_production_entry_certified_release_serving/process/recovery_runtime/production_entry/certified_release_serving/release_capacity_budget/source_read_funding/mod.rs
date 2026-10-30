//! Selector and addressed-root funding failures use genuine released, checkpointed Store media.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::release_capacity_budget::source_read_funding::*;
use crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family;
use worth_store_physical_format::ROOT_SELECTOR_BYTES;

#[test]
fn selector_and_root_reads_require_original_and_native_capacity_before_allocation() {
    std::thread::Builder::new()
        .name("release-source-read-funding".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            let store = world.serving().store_identity();
            let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
                worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xd5; 32]),
                worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
            );
            let worth_proof::TransitionOutcome::Success(handle) =
                world.serving().checkpoints().start(request).into_raw()
            else {
                panic!("real released world must admit its covering checkpoint");
            };
            let worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(checkpoint) = handle.wait() else {
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
                    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current(),
                ),
                (
                    24 << 20,
                    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current()
                        .with_residency_policy(tiny_recovery_policy())
                        .expect("tiny Recovery policy retains the actual configured format"),
                ),
            ];
            for (index, (memory, configuration)) in configurations.into_iter().enumerate() {
                let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
                    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_configuration(
                        root,
                        memory,
                        configuration,
                    ),
                );
                let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
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
                let [worth_store_recovery_runtime::PhysicalRecoverySourceDenial::SourceReadAllocation {
                    artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                    boundary: worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary::ReadBuffer,
                    requested,
                    cause: worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationDenial::Residency(cause),
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
                        worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
                    ) => {
                        assert_eq!(*required, ROOT_SELECTOR_BYTES as u64);
                        assert_eq!(*admitted, memory);
                    }
                    (1, worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::OperationAllocation(allocation)) => {
                        let pressure = allocation
                            .pressure()
                            .expect("actual native Recovery scope pressure");
                        assert_eq!(pressure.basis().store_identity(), store);
                        assert_eq!(pressure.scope(), worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
                        assert_eq!(
                            pressure.dimension(),
                            worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery)
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
            let selector = worth_store_physical_format::DurableRootSelector::decode(
                &std::fs::read(records.join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name()))
                    .expect("actual current selector remains readable after funding denials"),
            )
            .expect("actual current selector identifies its addressed root");
            let generation = selector.root_generation();
            let root_bytes = std::fs::metadata(
                records
                    .join("roots")
                    .join(worth_store_physical_format::RecordArtifactFile::RootManifest { generation }.file_name()),
            )
            .expect("actual addressed root remains present")
            .len();
            let selector_bytes = ROOT_SELECTOR_BYTES as u64;
            let scratch_bytes =
                worth_store_physical_format::DurablePhysicalRootManifest::maximum_encoding_scratch_bytes() as u64;
            for (boundary, requested, required, observed) in [
                (
                    worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary::ReadBuffer,
                    root_bytes,
                    selector_bytes + root_bytes,
                    selector_bytes,
                ),
                (
                    worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary::CanonicalValidation,
                    scratch_bytes,
                    selector_bytes + root_bytes + scratch_bytes,
                    selector_bytes + root_bytes,
                ),
            ] {
                let memory = required - 1;
                let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
                    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_configuration(
                        root,
                        memory,
                        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current(),
                    ),
                );
                let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
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
                let [worth_store_recovery_runtime::PhysicalRecoverySourceDenial::SourceReadAllocation {
                    artifact:
                        worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentRoot {
                            generation: denied_generation,
                        },
                    boundary: denied_boundary,
                    requested: denied_requested,
                    cause:
                        worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationDenial::Residency(worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
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

            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
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
                .certification_release_media_into_checkpoint_custody()
                .expect("independent Store rejoin seals real selected custody");
            let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal);
            assert_eq!(serving.store_identity(), store);
            assert_eq!(snapshot_family(root), before);
            assert!(!serving.close().residency().requires_inspection());
        })
        .expect("source read funding worker")
        .join()
        .expect("source read funding worker did not panic");
}

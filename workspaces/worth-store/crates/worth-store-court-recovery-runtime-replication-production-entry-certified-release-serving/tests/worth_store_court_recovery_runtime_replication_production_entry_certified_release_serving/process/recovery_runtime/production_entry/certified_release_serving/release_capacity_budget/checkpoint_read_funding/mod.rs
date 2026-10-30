//! Read/parser/source-root pressure uses the same genuinely released checkpoint as healthy rejoin.

use crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family;

#[test]
fn checkpoint_read_and_source_root_funding_deny_before_effects_and_retry() {
    std::thread::Builder::new()
        .name("checkpoint-read-funding".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
                worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xd6; 32]),
                worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
            );
            let worth_proof::TransitionOutcome::Success(handle) = world.serving().checkpoints().start(request).into_raw() else {
                panic!("real partial release must admit checkpoint");
            };
            assert!(matches!(handle.wait(), worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)));
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let before = snapshot_family(root);
            let checkpoint_bytes = std::fs::metadata(root.join("families/checkpoint.current")).unwrap().len();
            let native_configuration = worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current().with_residency_policy(
                crate::production_entry::certified_release_serving::release_capacity_budget::source_read_funding::recovery_policy_with_scope(checkpoint_bytes - 1),
            ).unwrap();
            for (memory, expected_boundary, configuration) in [
                (checkpoint_bytes - 1, worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary::ReadBuffer, worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current()),
                (checkpoint_bytes, worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary::ParserRecords, worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current()),
                (24 << 20, worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary::ReadBuffer, native_configuration),
            ] {
                let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
                    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_configuration(
                        root, memory, configuration,
                    ),
                );
                let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
                    panic!("bounded recovery outcome: {outcome:?}");
                };
                assert_eq!(blocked.recovery_effects(), 0);
                let evidence = blocked.evidence();
                assert!(evidence.counters.current_root_integrity_admissions > 0, "funded selector/root setup must pass first");
                let [worth_store_recovery_runtime::PhysicalRecoverySourceDenial::CheckpointReadAllocation { artifact: worth_store::physical_runtime::RecoveryDiscoveryArtifact::CurrentCheckpoint, boundary, requested, cause }] = evidence.source_denials.as_slice() else {
                    panic!("exact checkpoint allocation boundary required: {:?}", evidence.source_denials);
                };
                assert_eq!(*boundary, expected_boundary);
                match cause {
                    worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationDenial::Observation(worth_store::physical_runtime::PhysicalRecoveryObservationAllocationDenial::Residency(worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted }))
                    | worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationDenial::Residency(worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted }) => {
                        assert_eq!(*admitted, memory);
                        assert_eq!(*required, checkpoint_bytes + if expected_boundary == worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary::ParserRecords { *requested } else { 0 });
                    }
                    worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationDenial::Observation(worth_store::physical_runtime::PhysicalRecoveryObservationAllocationDenial::Residency(worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::OperationAllocation(failure))) => {
                        let pressure = failure.pressure().expect("actual native scope pressure");
                        assert_eq!(pressure.limit(), checkpoint_bytes - 1);
                        assert_eq!(pressure.requested(), checkpoint_bytes);
                        assert!(!pressure.effect_may_have_started());
                        assert_eq!(pressure.admitted(), 0, "the earlier root read window has ended");
                        assert_eq!(pressure.dimension(), worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery));
                    }
                    other => panic!("original native ceiling cause lost: {other:?}"),
                }
                assert!(*requested > 0);
                if expected_boundary == worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary::ReadBuffer {
                    assert_eq!(*requested, checkpoint_bytes);
                    assert_eq!(evidence.counters.checkpoint_integrity_attempts, 0);
                } else {
                    assert_eq!(evidence.integrity_counters().owner_projection_entries,
                        evidence.counters.current_root_candidate_interpretations + evidence.counters.previous_root_candidate_interpretations,
                        "only earlier root projections precede the checkpoint parser denial");
                }
                assert_eq!(snapshot_family(root), before);
            }
            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("healthy retry must recover identical real media: {outcome:?}");
            };
            let seal = handoff.into_core().certification_release_media_into_checkpoint_custody().expect("independent Store rejoin");
            let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal);
            assert!(!serving.close().residency().requires_inspection());
        })
        .unwrap()
        .join()
        .unwrap();
}

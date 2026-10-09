//! Read/parser/source-root pressure uses the same genuinely released checkpoint as healthy rejoin.

use super::*;
use worth_store::physical_runtime::{
    PhysicalRecoveryObservationAllocationDenial, RecoveryDiscoveryArtifact,
};
use worth_store_recovery_runtime::{
    PhysicalRecoverySourceDenial, PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};

#[test]
fn checkpoint_read_and_source_root_funding_deny_before_effects_and_retry() {
    std::thread::Builder::new()
        .name("checkpoint-read-funding".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            let request = PhysicalCheckpointRequest::fuzzy(
                PhysicalCheckpointIdempotencyKey::new([0xd6; 32]),
                PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
            );
            let TransitionOutcome::Success(handle) = world.serving().checkpoints().start(request).into_raw() else {
                panic!("real partial release must admit checkpoint");
            };
            assert!(matches!(handle.wait(), PhysicalCheckpointOutcome::Completed(_)));
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let before = snapshot_family(root);
            let checkpoint_bytes = fs::metadata(root.join("families/checkpoint.current")).unwrap().len();
            let native_configuration = PhysicalRecoveryStaticConfiguration::current().with_residency_policy(
                super::source_read_funding::recovery_policy_with_scope(checkpoint_bytes - 1),
            ).unwrap();
            for (memory, expected_boundary, configuration) in [
                (checkpoint_bytes - 1, Boundary::ReadBuffer, PhysicalRecoveryStaticConfiguration::current()),
                (checkpoint_bytes, Boundary::ParserRecords, PhysicalRecoveryStaticConfiguration::current()),
                (24 << 20, Boundary::ReadBuffer, native_configuration),
            ] {
                let outcome = WorthStoreRecovery::recover(
                    super::super::recovery_request::request_with_configuration(
                        root, memory, configuration,
                    ),
                );
                let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
                    panic!("bounded recovery outcome: {outcome:?}");
                };
                assert_eq!(blocked.recovery_effects(), 0);
                let evidence = blocked.evidence();
                assert!(evidence.counters.current_root_integrity_admissions > 0, "funded selector/root setup must pass first");
                let [PhysicalRecoverySourceDenial::CheckpointReadAllocation { artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint, boundary, requested, cause }] = evidence.source_denials.as_slice() else {
                    panic!("exact checkpoint allocation boundary required: {:?}", evidence.source_denials);
                };
                assert_eq!(*boundary, expected_boundary);
                match cause {
                    Cause::Observation(PhysicalRecoveryObservationAllocationDenial::Residency(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted }))
                    | Cause::Residency(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted }) => {
                        assert_eq!(*admitted, memory);
                        assert_eq!(*required, checkpoint_bytes + if expected_boundary == Boundary::ParserRecords { *requested } else { 0 });
                    }
                    Cause::Observation(PhysicalRecoveryObservationAllocationDenial::Residency(PhysicalRecoveryRejoinResidentDenial::OperationAllocation(failure))) => {
                        let pressure = failure.pressure().expect("actual native scope pressure");
                        assert_eq!(pressure.limit(), checkpoint_bytes - 1);
                        assert_eq!(pressure.requested(), checkpoint_bytes);
                        assert!(!pressure.effect_may_have_started());
                        assert_eq!(pressure.admitted(), 0, "the earlier root read window has ended");
                        assert_eq!(pressure.dimension(), worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(PhysicalOperationAllocationScope::Recovery));
                    }
                    other => panic!("original native ceiling cause lost: {other:?}"),
                }
                assert!(*requested > 0);
                if expected_boundary == Boundary::ReadBuffer {
                    assert_eq!(*requested, checkpoint_bytes);
                    assert_eq!(evidence.counters.checkpoint_integrity_attempts, 0);
                } else {
                    assert_eq!(evidence.integrity_counters().owner_projection_entries,
                        evidence.counters.current_root_candidate_interpretations + evidence.counters.previous_root_candidate_interpretations,
                        "only earlier root projections precede the checkpoint parser denial");
                }
                assert_eq!(snapshot_family(root), before);
            }
            let outcome = WorthStoreRecovery::recover(super::super::request(root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("healthy retry must recover identical real media: {outcome:?}");
            };
            let seal = handoff.into_core().into_checkpoint_custody().expect("independent Store rejoin");
            let serving = super::super::admit_serving_with_seal(root, seal);
            assert!(!serving.close().residency().requires_inspection());
        })
        .unwrap()
        .join()
        .unwrap();
}

//! Unchecked kernels use lease-free System execution; funded requests fail closed.
use super::custom_rules::AlwaysViolatesCustomRule;
use super::validation_engine_fixtures::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(posture: ExecutionPosture, workers: usize) -> ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                posture,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(workers).unwrap(),
                    8 * 1024 * 1024,
                    100_000,
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

#[test]
fn an_unchecked_custom_kernel_is_denied_by_every_leased_posture() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(AlwaysViolatesCustomRule).unwrap())
        .build();
    let run = |execution: &ExecutionRequest<'_, '_>| {
        let request = InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        );
        InvariantEngine::new(&runtime).execute(execution, request)
    };
    let serial_request = crate::test_execution_policy::serial_request();
    let expected = run(&ExecutionRequest::serial(&serial_request)).unwrap();
    assert!(matches!(
        expected.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
    for (posture, workers) in [
        (ExecutionPosture::Serial, 4),
        (ExecutionPosture::Automatic, 1),
        (ExecutionPosture::Automatic, 2),
    ] {
        runtime.performance_access().reset_counters();
        let lease = lease(posture, workers);
        assert!(matches!(
            run(&ExecutionRequest::leased(&lease)),
            Err(
                crate::transactions::data::TransactionCommitError::Execution {
                    denial: crate::transactions::data::CommitExecutionDenial {
                        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                            crate::execution::RelationalExecutionDenialCause::UncheckedCustomKernel
                        ),
                        partition_identity: None,
                    },
                    ..
                }
            )
        ));
        assert_eq!(
            runtime
                .performance_access()
                .counters()
                .custom_invariant_execution_count,
            0
        );
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

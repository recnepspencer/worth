//! A kernel needing no parallel contract remains callable on a serial backing.
use super::custom_rules::AlwaysViolatesCustomRule;
use super::validation_engine_fixtures::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(posture: ExecutionPosture, workers: usize) -> ExecutionResourceLease<'static> {
    crate::execution::test_authority::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                posture,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), None, Some(100_000)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

#[test]
fn an_unchecked_custom_kernel_runs_only_when_the_request_cannot_run_in_parallel() {
    let instrumentation = crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::identity::data::VersionId(0);
    let mut config = crate::config::data::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(AlwaysViolatesCustomRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();
    let run = |execution: &ExecutionRequest<'_, '_>| {
        let request = InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            None,
            None,
        );
        InvariantEngine::from_view(&view).execute(execution, request)
    };
    let serial_request = crate::execution::test_policy::serial_request();
    let expected = run(&ExecutionRequest::serial(&serial_request)).unwrap();
    for (posture, workers) in [
        (ExecutionPosture::Serial, 4),
        (ExecutionPosture::Automatic, 1),
    ] {
        let lease = lease(posture, workers);
        let actual = run(&ExecutionRequest::leased(&lease))
            .expect("a serial-backed request can call the existing custom kernel");
        assert_eq!(actual.results(), expected.results());
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    let parallel = lease(ExecutionPosture::Automatic, 2);
    assert!(matches!(
        run(&ExecutionRequest::leased(&parallel)),
        Err(
            crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
                denial: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                    kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::UncheckedCustomKernel
                    ),
                    partition_identity: None,
                },
                ..
            }
        )
    ));
}

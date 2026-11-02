//! Original request limits veto publication even when the owner masks a stop.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionAuthority, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial;

#[test]
fn domain_write_admission_keeps_original_work_refusal() {
    run(17, 500_000, 32 * 1024 * 1024, true);
}
#[test]
fn domain_write_admission_keeps_original_memory_refusal() {
    run(19, 2_000_000, 4 * 1024 * 1024, false);
}
fn run(value: u64, work: u64, memory: u64, work_stop: bool) {
    let application = host::install();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let before = super::super::super::model::required_output_courts::head(&application);
    let execute = |work, memory, key: &str| {
        let lease = ExecutionAuthority::serial()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, Some(memory), Some(work)),
                ),
                deadline: scope.deadline(),
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        runtime
            .request(&principal, &scope)
            .advanced()
            .mutate(Write {
                key: "numeric".into(),
                value,
                encoding: Encoding::Binary,
            })
            .without_source()
            .idempotency(&key.to_owned())
            .execute_in_program(&application, ExecutionAllocationPolicy::Execution(&lease))
            .unwrap()
    };
    let outcome = execute(work, memory, "bounded-domain-write");
    assert_eq!(
        super::super::super::model::required_output_courts::head(&application),
        before,
        "refused request published: {outcome:?}"
    );
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("expected original resource denial: {outcome:?}")
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::EffectLowering
    );
    assert!(
        denial.domain_value_decode_denial().is_none(),
        "owner's Malformed must not hide original stop"
    );
    let Some(Ok(WorthQueryProviderSessionDenialKind::ExecutionResource {
        denial: resource, ..
    })) = denial.execution_denial_cause()
    else {
        panic!("missing typed original resource cause: {denial:?}")
    };
    if work_stop {
        assert_eq!(
            resource,
            WorthQueryManagedComputationResourceDenial::WorkExhausted
        );
    } else {
        assert!(
            matches!(
                resource,
                WorthQueryManagedComputationResourceDenial::MemoryLimit { .. }
            ),
            "{resource:?}"
        );
    }
    let retry = execute(2_000_000, 32 * 1024 * 1024, "funded-domain-write");
    assert!(
        matches!(
            retry,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{retry:?}"
    );
    assert_ne!(
        super::super::super::model::required_output_courts::head(&application),
        before
    );
}

//! The real Host constructor separates original setup custody from World calls.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryAdvancementDenial as OpeningDenial,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    application_entry::WorthQueryApplicationRequestQueryDenial,
};

fn policy(memory: Option<u64>, work: u64) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, memory, Some(work)),
    )
}
fn runtime_limits() -> WorthQueryInMemoryApplicationLimits {
    limits_with_policy(
        DEFAULT_RETAINED_COMMITS,
        16,
        64,
        invalidation(),
        policy(None, 0),
    )
}

#[test]
fn funded_bootstrap_keeps_the_worlds_zero_work_query_refusal() {
    let _guard = checkpoint_recovery_test_guard();
    let cancellation = CancellationSource::new();
    let setup = SerialRequest::from_policy(&policy(None, 8_000_000), cancellation.token(), None);
    let application =
        with_bootstrap_advancement_from_request(ExecutionRequest::serial(&setup), |phase| {
            install_program_in_bootstrap::<CheckpointProgram>(
                &phase,
                Default::default(),
                runtime_limits(),
                seed_cycle,
            )
        })
        .unwrap()
        .unwrap();
    assert!(
        setup.memory().unwrap().charged_memory_bytes() > 0,
        "the installed canonical backing retains its original setup payer"
    );
    {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let Err(denial) = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
        else {
            panic!("funding setup must not fund a zero-work runtime query");
        };
        assert!(matches!(
            denial,
            WorthQueryApplicationRequestQueryDenial::ExecutionRequest(OpeningDenial::Resource(
                Resource::WorkExhausted
            ))
        ));
    }
    drop(application);
    assert_eq!(setup.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn fresh_installation_refuses_the_original_cancelled_phase_before_seed() {
    let _guard = checkpoint_recovery_test_guard();
    let cancellation = CancellationSource::new();
    let setup = SerialRequest::from_policy(&policy(None, 8_000_000), cancellation.token(), None);
    let mut seeded = false;
    let result =
        with_bootstrap_advancement_from_request(ExecutionRequest::serial(&setup), |phase| {
            cancellation.cancel();
            install_program_in_bootstrap::<CheckpointProgram>(
                &phase,
                Default::default(),
                runtime_limits(),
                |_| seeded = true,
            )
        })
        .unwrap();
    let Err(denial) = result else {
        panic!("the original cancellation must refuse installation")
    };
    let application_installation::WorthQueryInMemoryApplicationDenial::Graph(cause) = *denial
    else {
        panic!("entry preserves its typed original request refusal");
    };
    assert!(matches!(
        cause.kind(),
        primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
            kind: primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled
        }
    ));
    assert!(!seeded);
    assert_eq!(setup.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn zero_memory_bootstrap_refuses_without_borrowing_the_worlds_resources() {
    let _guard = checkpoint_recovery_test_guard();
    let setup = SerialRequest::from_policy(
        &policy(Some(0), 8_000_000),
        CancellationSource::new().token(),
        None,
    );
    let mut seeded = false;
    let result =
        with_bootstrap_advancement_from_request(ExecutionRequest::serial(&setup), |phase| {
            install_program_in_bootstrap::<CheckpointProgram>(
                &phase,
                Default::default(),
                runtime_limits(),
                |_| seeded = true,
            )
        });
    assert!(matches!(
        result,
        Err(OpeningDenial::Resource(Resource::PolicyMemoryLimit))
    ));
    assert!(!seeded);
    assert_eq!(setup.memory().unwrap().charged_memory_bytes(), 0);
}

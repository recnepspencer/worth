//! Exact pre-seal alias refusal followed by genuine ordinary-handler close.
use super::*;
use worth_foundational::facade::{
    DeterminismContract, EquivalenceContractId, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeOpenRefused,
    WorthQueryHomeClosePolicyRefused,
};

#[inline(never)]
pub(super) fn verify(
    home: WorthQueryApplicationHome<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    policy: ExecutionRequestPolicy,
) {
    assert!(
        std::mem::size_of::<
            WorthQueryApplicationHomeOpenRefused<
                fixture::schema::HomeIdentitySchema,
                fixture::HomeProgram,
            >,
        >() <= 16,
        "WorthQueryApplicationHomeOpenRefused must remain pointer-sized to avoid result_large_err"
    );
    let equivalent = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::ContractEquivalent(EquivalenceContractId::new(1)),
        std::num::NonZeroUsize::MIN,
    );
    let Err(WorthQueryHomeClosePolicyRefused::Admission {
        owner: home,
        cause: worth_execution::LeaseDenial::EquivalenceContractUnavailable,
    }) = home.close_with_policy(equivalent)
    else {
        panic!("a serial lease without an equivalence registry returns the original Home");
    };
    let home = *home;
    let mut retained_runtime = home.runtime().clone();
    let refused = home
        .close(execution)
        .err()
        .expect("the actual retained Query runtime refuses before World seal");
    use worth_query_host::facade::primary_graph::{
        WorthQueryHomeRetirementDenial as CloseDenial, WorthQueryHomeRetirementOwner as CloseOwner,
    };
    assert!(
        matches!(
            refused.query_denial(),
            Some(CloseDenial::Busy(CloseOwner::ProgramRuntime))
        ),
        "the exact aliased owner refuses before seal: {refused:?}"
    );
    assert!(refused.directory_denial().is_none());
    {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
        let principal = identity::inspection_authentication::authenticate(
            retained_runtime.runtime().installed_schema(),
            &scope,
        );
        let observed = retained_runtime
            .request(&principal, &scope)
            .query(fixture::balance_query::AccountRead(
                "account-001".to_owned(),
            ))
            .execute()
            .expect("pre-seal refusal preserves the original serving Home");
        assert_eq!(observed.rows(), &[8]);
    }
    let legacy_denial = match retained_runtime.close_conditional_runtime() {
        Ok(_) => panic!("the retained Query clone never acquires Store lifecycle custody"),
        Err(denial) => denial,
    };
    assert_eq!(
        legacy_denial.kind(),
        WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained,
    );
    drop(retained_runtime);
    let closed = refused
        .try_close_with_policy(policy)
        .unwrap_or_else(|refused| {
            panic!("ordinary handler retires after alias release: {refused:?}")
        });
    assert!(closed.is_closed());
    closed.reclaim();
}

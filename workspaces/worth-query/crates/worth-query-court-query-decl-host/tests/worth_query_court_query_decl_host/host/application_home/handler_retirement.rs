//! Original aliases refuse before seal; ordinary handlers retire after service.
use super::*;
type OrdinaryHome = worth_query_host::facade::WorthQueryApplicationHome<
    fixture::schema::HomeIdentitySchema,
    fixture::HomeProgram,
>;
use worth_query_host::facade::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
};

#[test]
fn program_alias_refusal_resumes_the_same_query_and_ordinary_handler_retires() {
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = resume_original_query_after_alias_refusal(
        &execution,
        initialized.clone(),
        contacts.clone(),
    );
    execute_original_mutation_after_query_alias_release(&home);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("ordinary handler close: {refused:?}"));
    assert!(closed.is_closed());
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        1,
        "retirement never invokes business"
    );
    closed.reclaim();
}

#[inline(never)]
fn resume_original_query_after_alias_refusal(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    initialized: Arc<AtomicUsize>,
    contacts: Arc<AtomicUsize>,
) -> OrdinaryHome {
    let request = fixture::request(
        fixture::location(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        initialized,
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(*execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("genuine Directory opening: {refused:?}"));
    let retained = home.runtime().clone();
    let refused = home
        .close(&execution)
        .err()
        .expect("the original backing is shared");
    assert!(matches!(
        refused.query_denial(),
        Some(Denial::Busy(Owner::ProgramRuntime))
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    let home = refused.resume().unwrap();
    drop(retained);
    home
}

#[inline(never)]
fn execute_original_mutation_after_query_alias_release(home: &OrdinaryHome) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    {
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let outcome = runtime
            .request(&principal, &scope)
            .advanced()
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".to_owned(),
                balance: 8,
            })
            .without_source()
            .idempotency(&0x443131_u64)
            .execute_in_program(
                runtime,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("the same original Query executes after its original alias refusal");
        assert!(matches!(outcome, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }));
        let observed = runtime
            .request(&principal, &scope)
            .query(fixture::balance_query::AccountRead(
                "account-001".to_owned(),
            ))
            .execute()
            .unwrap();
        assert_eq!(observed.rows(), &[8]);
    }
}

#[test]
fn handler_declared_refusal_preserves_the_same_serving_query_until_supplier_release() {
    use std::sync::atomic::AtomicBool;
    use worth_query_decl::facade::application_operation::ApplicationMutationBinding;
    use worth_query_host::facade::primary_graph::WorthQueryOperationHomeRetirementDenial as HandlerDenial;
    let contacts = Arc::new(AtomicUsize::new(0));
    let refusal = Arc::new(AtomicBool::new(true));
    let request = fixture::request_with_handler_retirement_refusal(
        fixture::location(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        contacts.clone(),
        refusal.clone(),
    );
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("genuine Directory opening: {refused:?}"));
    let refused = home
        .close(&execution)
        .err()
        .expect("the supplier explicitly refuses");
    assert!(matches!(refused.query_denial(),
        Some(Denial::Handler { binding, cause: HandlerDenial::Unsupported })
            if *binding == fixture::native_mutation::SetBalanceBinding::IDENTITY));
    let home = refused.resume().unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    {
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let observed = runtime
            .request(&principal, &scope)
            .query(fixture::balance_query::AccountRead(
                "account-001".to_owned(),
            ))
            .execute()
            .unwrap();
        assert_eq!(observed.rows(), &[7]);
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    refusal.store(false, Ordering::SeqCst);
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("same supplier released: {refused:?}"));
    assert!(closed.is_closed());
    closed.reclaim();
}

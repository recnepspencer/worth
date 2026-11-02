//! Real pre-seal Query refusal preserves its original Directory service.
use super::*;
use worth_query_decl::facade::application_operation::ApplicationMutationBinding;
use worth_query_host::facade::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementDenial as HandlerDenial,
};

#[test]
fn original_program_alias_and_opaque_handler_refusals_resume_the_same_directory_query() {
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let request = fixture::request(
        fixture::location(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        initialized.clone(),
        contacts.clone(),
    );
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
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

    let refused = home
        .close(&execution)
        .err()
        .expect("opaque supplier retirement is unsupported");
    assert!(matches!(
        refused.query_denial(),
        Some(Denial::Handler { binding, cause: HandlerDenial::Unsupported })
            if *binding == fixture::native_mutation::SetBalanceBinding::IDENTITY
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    let home = refused.resume().unwrap();
    assert!(home.is_directory());
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
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
            .expect("the same original Query executes after both pre-seal refusals");
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
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    // Remaining Native/Bridge source retirement is unfinished. Ordinary owner
    // abandonment here is unbudgeted and grants no physical Closed evidence.
    drop(home);
}

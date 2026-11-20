//! A cancelled adoption's Native-only prefix refuses close, then continues
//! after an unclean reopen.
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
    },
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationProgramRoster, WorthQueryProgramOwner,
    },
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, seed_host},
    model_lifecycle::validated_model_lifecycle_program,
    operator_identity::authenticate_operator,
    presented_request::set_retention,
    programs::{validated_first_program, validated_second_program, RetentionProgramP1},
    readback::read_retention,
    schema::DocumentRetentionSchema,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn native_only_cancelled_adoption_refuses_close_and_continues_after_unclean_reopen() {
    use worth_query_host::facade::{
        admission::authenticated_principal::{
            WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
        },
        application_entry::WorthQueryBranchAdoptionRecoveryOutcome,
        ApplicationHomeCloseDenial, RuntimeWorldCloseDenial, RuntimeWorldServiceDenial,
    };
    let directory = worth_query_testkit_host::facade::host::application_home::location();
    let lease = worth_query_testkit_host::facade::host::checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let open = |initialize: bool| {
        let request = WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            validated_first_program(),
            WorthQueryApplicationProgramRoster::new()
                .support(validated_second_program())
                .support(validated_actor_program())
                .support(validated_model_lifecycle_program()),
            DocumentRetentionSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            host_limits(WorthQueryAdmissionClock::system()),
            move |phase, graph, schema| {
                assert!(initialize);
                seed_host(phase, graph, schema)
            },
        );
        with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase).map(Box::new)
        })
        .unwrap()
    };
    let home = open(true).unwrap();
    let target = {
        let host = home.runtime();
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
            cancellation.token(),
            WorthQueryAdmissionClock::system(),
        );
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let target = *host
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .owned_revision();
        let request = host.runtime().request(&principal, &scope);
        let requirements = request
            .on_branch(host.current_world())
            .programs()
            .compare(&target)
            .unwrap();
        let prepared = host
            .runtime()
            .request(&principal, &scope)
            .on_branch(host.current_world())
            .programs()
            .adopt(&requirements)
            .unwrap()
            .prepare(64)
            .unwrap();
        let pause = host
            .runtime()
            .world_operation_control_for_test()
            .pause_before_product_compare_for_test(std::num::NonZeroUsize::MIN);
        let outcome = std::thread::scope(|threads| {
            let publisher = threads.spawn(|| prepared.publish(host.runtime(), &principal, &scope));
            assert!(pause.wait_until_reached(std::time::Duration::from_secs(10)));
            cancellation.cancel();
            pause.release();
            publisher.join().unwrap()
        });
        let WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) = outcome
        else {
            panic!("actual Native effect must remain an unpublished original");
        };
        let retained = unpublished.into_recovery();
        let page = host
            .runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
            .unwrap();
        let handle = page.rows()[0].handle();
        let original = host
            .runtime()
            .readmit_product_publication_recovery(handle)
            .unwrap();
        let inspected = original.inspect().unwrap();
        assert!(
            inspected.native_semantic_result().is_some(),
            "real acknowledged Native effect"
        );
        drop(inspected);
        drop(original);
        drop(retained);
        target
    };
    let refused = (*home)
        .close(&execution)
        .err()
        .expect("Native-only prefix cannot close");
    assert!(matches!(
        refused.directory_denial(),
        Some(ApplicationHomeCloseDenial::World(
            RuntimeWorldServiceDenial::Denied(RuntimeWorldCloseDenial::UnresolvedBusinessCustody)
        ))
    ));
    let home = refused
        .resume()
        .unwrap_or_else(|_| panic!("pre-seal close refusal retains usable Home"));
    assert_eq!(
        read_retention(home.runtime().runtime(), home.runtime().current_world()),
        7
    );
    assert!(!home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    drop(home);
    let home = open(false).unwrap_or_else(|refused| panic!("unclean reopen recovers: {refused:?}"));
    let host = home.runtime();
    assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
    let page = host
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let recovery = host
        .runtime()
        .readmit_branch_adoption(page.rows()[0].handle())
        .unwrap_or_else(|cause| panic!("the same adoption original readmits: {cause:?}"));
    assert_eq!(recovery.target(), &target);
    drop(page);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("the original adoption continues"));
    let WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } = outcome else {
        panic!("one recovered adoption publishes");
    };
    assert_eq!(adoption.target(), &target);
    cleanup.unwrap_or_else(|cause| panic!("recovered custody releases: {cause:?}"));
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(
        settle(set_retention(host, host.current_world(), 15, 1)),
        RetentionVerdict::Performed(15),
        "ordinary calls use the adopted rule"
    );
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("close after continuation: {cause:?}"))
        .reclaim();
    let home = open(false).unwrap();
    let host = home.runtime();
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(read_retention(host.runtime(), host.current_world()), 15);
}

//! C1 retains the Native-only prefix; H refuses close and unclean reopen.
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
    programs::{validated_first_program, validated_second_program, RetentionProgramP1},
    readback::read_retention,
    schema::DocumentRetentionSchema,
};

#[test]
fn native_only_cancelled_adoption_refuses_close_and_d18_reopen() {
    use worth_query_host::facade::{
        admission::authenticated_principal::{
            WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
        },
        ApplicationHomeCloseDenial, DurableOwnerFamily, RuntimeWorldCloseDenial,
        RuntimeWorldServiceDenial,
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
    {
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
    }
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
    let refused = open(false)
        .err()
        .expect("D18 rejects associated Native effect without World occurrence");
    let unclean = refused
        .unclean_recovery_unsupported()
        .unwrap_or_else(|| panic!("typed D18 posture: {refused:?}"));
    assert_eq!(unclean.owner, DurableOwnerFamily::Relational);
}

//! Graceful adoption stages preserve activation and discharge source custody.
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
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    programs::{validated_first_program, validated_second_program, RetentionProgramP1},
    readback::read_retention,
    schema::DocumentRetentionSchema,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn directory_adoption_survives_graceful_assessment_and_completed_publication() {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
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
            Box::new(request.open_in_advancement(&phase).unwrap())
        })
        .unwrap()
    };
    let home = open(true);
    {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let target = *host
            .supported_program::<RetentionProgramP1>()
            .expect("target support after graceful assessment reopen")
            .owned_revision();
        let request = host.runtime().request(&principal, &scope);
        let _requirements = request
            .on_branch(host.current_world())
            .programs()
            .compare(&target)
            .unwrap();
    }
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("pre-effect close: {cause:?}"))
        .reclaim();
    let home = open(false);
    {
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let target = *host
            .supported_program::<RetentionProgramP1>()
            .expect("target support after graceful assessment reopen")
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
        let WorthQueryBranchAdoptionPublicationOutcome::Performed(adoption) =
            prepared.publish(host.runtime(), &principal, &scope)
        else {
            panic!("one actual adoption must publish");
        };
        assert_eq!(adoption.target(), &target);
        assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
    }
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("post-occurrence close: {cause:?}"))
        .reclaim();
    let home = open(false);
    let host = home.runtime();
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(
        settle(set_retention(host, host.current_world(), 15, 1)),
        RetentionVerdict::Performed(15)
    );
    assert_eq!(
        read_retention(host.runtime(), host.current_world()),
        15,
        "ordinary calls use the adopted rule"
    );
    // The initial program remains rostered metadata; source custody no longer
    // pins it after its exact adoption has completed.
    let source = *host.owned_revision();
    host.retire_program_support(&execution, &source).unwrap();
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("terminal adoption close: {cause:?}"))
        .reclaim();
    let home = open(false);
    let host = home.runtime();
    assert!(
        host.presented_program().is_none(),
        "released adoption custody permits durable source retirement"
    );
    assert_eq!(read_retention(host.runtime(), host.current_world()), 15);
}

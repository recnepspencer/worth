//! Build identity through ordinary Directory Home close and readmission.
use super::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program, host::host_limits,
    model_lifecycle::validated_model_lifecycle_program, schema::DocumentRetentionSchema,
};
#[test]
fn directory_changed_build_reuses_original_output_until_new_implementation_is_demanded() {
    use worth_query_host::facade::{
        application_contribution::with_bootstrap_advancement_from_request,
        WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
    };
    use worth_query_testkit_document_retention::facade::document_retention_model::host::seed_host;
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let directory = worth_query_testkit_host::facade::host::application_home::location();
    let build_a = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let build_b = ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap();
    assert_ne!(build_a, build_b);
    let lease = worth_query_testkit_host::facade::host::checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let open = |build: ApplicationBuildIdentity, initialize: bool| {
        let request = WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            validated_program(),
            WorthQueryApplicationProgramRoster::new()
                .support(validated_second_program())
                .support(validated_actor_program())
                .support(validated_model_lifecycle_program()),
            DocumentRetentionSchema::declaration().unwrap(),
            ((),),
            build,
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
            move |phase, graph, schema| {
                assert!(initialize, "an existing producer Home cannot initialize");
                seed_host(phase, graph, schema)
            },
        );
        with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(
                request
                    .open_in_advancement(&phase)
                    .unwrap_or_else(|cause| panic!("producer Home readmission: {cause:?}")),
            )
        })
        .unwrap()
    };
    let home = open(build_a, true);
    let host = home.runtime();
    assert_eq!(
        settle(set_retention(host, host.current_world(), 6, 971342)),
        RetentionVerdict::Performed(6)
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let mut demand = request
        .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .start_in_program::<_, AssessmentRoot>(host)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(original) =
        demand.settle(&request).unwrap()
    else {
        panic!("the actual directory producer must settle");
    };
    assert_eq!(original.producer_contacts_in_this_demand(), 1);
    let original_id = original
        .outputs_of::<RetentionAssessmentOutputs>()
        .unwrap()
        .entity::<AssessmentOutput>()
        .unwrap()
        .entity_id();
    assert_eq!(
        host.runtime().producer_contacts_on_this_thread_for_test(),
        1
    );
    drop(original);
    drop(demand);
    drop(request);
    drop(principal);
    drop(scope);
    drop(home); // Quiescent teardown, as in the cited track-H court.

    let home = open(build_a, false);
    assert_eq!(
        home.runtime()
            .runtime()
            .producer_contacts_on_this_thread_for_test(),
        0,
        "same-build open cannot contact the producer"
    );
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let mut demand = request
        .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .start_in_program::<_, AssessmentRoot>(host)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(reused) =
        demand.settle(&request).unwrap()
    else {
        panic!("same-build durable output must remain reusable");
    };
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(
        reused
            .outputs_of::<RetentionAssessmentOutputs>()
            .unwrap()
            .entity::<AssessmentOutput>()
            .unwrap()
            .entity_id(),
        original_id
    );
    assert_eq!(
        host.runtime().producer_contacts_on_this_thread_for_test(),
        0
    );
    drop(reused);
    drop(demand);
    drop(request);
    drop(principal);
    drop(scope);
    drop(home); // Quiescent teardown, as in the cited track-H court.

    let home = open(build_b, false);
    assert_eq!(
        home.runtime()
            .runtime()
            .producer_contacts_on_this_thread_for_test(),
        0,
        "changed-build open cannot eagerly regenerate output"
    );
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let mut demand = request
        .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .start_in_program::<_, AssessmentRoot>(host)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(refreshed) =
        demand.settle(&request).unwrap()
    else {
        panic!("explicit demand must reproduce the stale producing implementation");
    };
    assert_eq!(refreshed.producer_contacts_in_this_demand(), 1);
    let read = request.retain_current_read().unwrap();
    request
        .at(&read)
        .require_current_output_demand(&refreshed)
        .unwrap();
    assert_eq!(
        refreshed
            .outputs_of::<RetentionAssessmentOutputs>()
            .unwrap()
            .entity::<AssessmentOutput>()
            .unwrap()
            .entity_id(),
        original_id,
        "changed-build demand preserves the authored output identity"
    );
    assert_eq!(
        host.runtime().producer_contacts_on_this_thread_for_test(),
        1
    );
    drop(read);
    drop(refreshed);
    drop(demand);
    drop(request);
    drop(principal);
    drop(scope);
    drop(home); // Quiescent teardown, as in the cited track-H court.
}

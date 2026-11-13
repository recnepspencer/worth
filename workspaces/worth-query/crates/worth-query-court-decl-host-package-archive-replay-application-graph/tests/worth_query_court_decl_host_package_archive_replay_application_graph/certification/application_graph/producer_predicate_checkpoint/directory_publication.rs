//! Ordinary producer delivery uses its actual authored branch allocations.
use super::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program, host::host_limits,
    model_lifecycle::validated_model_lifecycle_program, schema::DocumentRetentionSchema,
};
#[test]
fn directory_ordinary_producer_settles_through_its_authored_allocation_registry() {
    use worth_query_host::facade::{
        application_contribution::with_bootstrap_advancement_from_request,
        WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
    };
    use worth_query_testkit_document_retention::facade::document_retention_model::host::seed_host;
    let directory = worth_query_testkit_host::facade::host::application_home::location();
    let build_a = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
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
    let _original_id = original
        .outputs_of::<RetentionAssessmentOutputs>()
        .unwrap()
        .entity::<AssessmentOutput>()
        .unwrap()
        .entity_id();
    drop(original);
    drop(demand);
    drop(request);
    drop(principal);
    drop(scope);
    drop(home); // Quiescent teardown, as in the cited track-H court.
}

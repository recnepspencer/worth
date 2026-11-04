//! Changed-build output lineage through the existing checkpoint composition.
//! This component court does not substitute for directory Host readmission.
use super::*;
use worth_query_host::facade::application_installation::{
    in_memory_rostered_program_from_checkpoint, ApplicationBuildIdentity,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program, host::host_limits,
    retention_entry::assessment_counts, schema::DocumentRetentionSchema,
};

#[test]
fn changed_build_preserves_recorded_output_and_reproduces_stale_output_only_on_demand() {
    let (checkpoint, original_output, accepted, contacts) = {
        let host = publish(
            validated_program(),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .unwrap();
        assert_eq!(
            settle(set_retention(&host, host.current_world(), 6, 971341)),
            RetentionVerdict::Performed(6)
        );
        let runtime = host.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(runtime.installed_schema(), &scope);
        let request = runtime.request(&principal, &scope);
        let mut demand = request
            .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
            .start_in_program::<_, AssessmentRoot>(&host)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.settle(&request).unwrap()
        else {
            panic!("the real producer must publish the original output");
        };
        assert_eq!(settled.producer_contacts_in_this_demand(), 1);
        let output = settled
            .outputs_of::<RetentionAssessmentOutputs>()
            .unwrap()
            .entity::<AssessmentOutput>()
            .unwrap()
            .entity_id();
        let (checkpoint, sections) = runtime
            .with_host_advancement(|phase| {
                runtime
                    .capture_application_checkpoint_with_sections(
                        &phase,
                        CapturePolicy::SystemAllocation,
                    )
                    .unwrap()
            })
            .unwrap();
        assert_eq!(sections.accepted_output_count(), 1);
        let accepted = checkpoint.bytes()
            [checkpoint.bytes().len() - sections.accepted_output_bytes()..]
            .to_vec();
        (checkpoint, output, accepted, assessment_counts(6))
    };
    let restored = in_memory_rostered_program_from_checkpoint(
        validated_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_actor_program()),
        DocumentRetentionSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap(),
        host_limits(),
        checkpoint,
    )
    .unwrap();
    assert_eq!(
        assessment_counts(6),
        contacts,
        "readmission must not execute either producer phase"
    );
    let runtime = restored.runtime();
    let (recaptured, sections) = runtime
        .with_host_advancement(|phase| {
            runtime
                .capture_application_checkpoint_with_sections(
                    &phase,
                    CapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(sections.accepted_output_count(), 1);
    assert_captured_document_predicate(
        recaptured.bytes(),
        sections.accepted_output_bytes(),
        original_output,
    );
    assert_eq!(
        &recaptured.bytes()[recaptured.bytes().len() - sections.accepted_output_bytes()..],
        accepted.as_slice(),
        "build B retains A's original producing implementation and authored output record"
    );
    assert_eq!(assessment_counts(6), contacts);

    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let mut demand = request
        .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .start_in_program::<_, AssessmentRoot>(&restored)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("the explicit demand must reproduce the stale output");
    };
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        1,
        "build A's recorded output cannot be reused by B"
    );
    assert!(settled.application_commit_receipt().is_some());
    let read = request.retain_current_read().unwrap();
    request
        .at(&read)
        .require_current_output_demand(&settled)
        .unwrap();
    assert_eq!(assessment_counts(6), (contacts.0 + 1, contacts.1 + 1));
}

#[test]
fn directory_changed_build_reuses_original_output_until_new_implementation_is_demanded() {
    use worth_query_host::facade::{
        application_contribution::with_bootstrap_advancement_from_request,
        WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
    };
    use worth_query_testkit_document_retention::facade::document_retention_model::host::seed_host;
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
                .support(validated_actor_program()),
            DocumentRetentionSchema::declaration().unwrap(),
            ((),),
            build,
            host_limits(),
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
    let contacts = assessment_counts(6);
    drop(original);
    drop(demand);
    drop(request);
    drop(principal);
    drop(scope);
    drop(home);

    let home = open(build_a, false);
    assert_eq!(
        assessment_counts(6),
        contacts,
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
    assert_eq!(assessment_counts(6), contacts);
    drop(reused);
    drop(demand);
    drop(request);
    drop(principal);
    drop(scope);
    drop(home);

    let home = open(build_b, false);
    assert_eq!(
        assessment_counts(6),
        contacts,
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
    assert_eq!(assessment_counts(6), (contacts.0 + 1, contacts.1 + 1));
}

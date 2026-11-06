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
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
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

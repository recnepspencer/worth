//! Changed-build output lineage through Directory Home close and readmission.
use super::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits, retention_entry::assessment_counts,
};

#[test]
fn changed_build_preserves_recorded_output_and_reproduces_stale_output_only_on_demand() {
    let build_a = ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap();
    let build_b = ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap();
    let (directory, original_output, contacts) = {
        let host = open_directory_with_build(
            unique_directory("producer-predicate-changed-build"),
            validated_program(),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
            build_a,
            host_limits(
                worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
            ),
        )
        .unwrap();
        let output = {
            assert_eq!(
                settle(set_retention(&host, host.current_world(), 9, 971341)),
                RetentionVerdict::Performed(9)
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
            settled
                .outputs_of::<RetentionAssessmentOutputs>()
                .unwrap()
                .entity::<AssessmentOutput>()
                .unwrap()
                .entity_id()
        };
        let directory = host.checkpoint_and_close_for_reopen();
        (directory, output, assessment_counts(9))
    };
    let restored = open_directory_with_build(
        directory,
        validated_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        build_b,
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
    )
    .unwrap();
    let runtime = restored.runtime();
    assert_eq!(
        assessment_counts(9),
        contacts,
        "readmission must not execute either producer phase"
    );
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
    assert_eq!(
        settled
            .outputs_of::<RetentionAssessmentOutputs>()
            .unwrap()
            .entity::<AssessmentOutput>()
            .unwrap()
            .entity_id(),
        original_output,
        "build B regenerates the derived assessment on the original authored document"
    );
    assert_eq!(assessment_counts(9), (contacts.0 + 1, contacts.1 + 1));
    let mut repeated = request
        .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .start_in_program::<_, AssessmentRoot>(&restored)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(reused) =
        repeated.settle(&request).unwrap()
    else {
        panic!("B's acknowledged output must be reusable by B");
    };
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(
        reused
            .outputs_of::<RetentionAssessmentOutputs>()
            .unwrap()
            .entity::<AssessmentOutput>()
            .unwrap()
            .entity_id(),
        original_output
    );
    request
        .at(&read)
        .require_current_output_demand(&reused)
        .unwrap();
    assert_eq!(assessment_counts(9), (contacts.0 + 1, contacts.1 + 1));
}

//! A real producer's handler predicate survives captured output readmission.
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationRequestExt,
    WorthQueryOutputCurrentnessDenial,
};
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;
use worth_relational::facade::identity::EntityId;

use super::document_retention_model::{
    assessment_output::{AssessmentOutput, RetentionAssessmentDemand, RetentionAssessmentOutputs},
    host::{publish, restore},
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    programs::validated_second_program,
    retention_entry::DOCUMENT_IDENTITY,
    settled_verdict::{settle, RetentionVerdict},
};

#[path = "producer_predicate_checkpoint/ordinary_source.rs"]
mod ordinary_source;
#[path = "producer_predicate_checkpoint/program.rs"]
mod program;
#[path = "producer_predicate_checkpoint/transition.rs"]
mod transition;
use program::{validated_program, AssessmentRoot};

#[test]
fn captured_producer_predicate_reuses_after_reopen_and_refreshes_after_source_change() {
    let (checkpoint, subject, foreign_settlement) = {
        let host = publish(
            validated_program(),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .unwrap();
        let runtime = host.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(runtime.installed_schema(), &scope);
        let request = runtime.request(&principal, &scope);
        let mut handle = request
            .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
            .start_in_program::<_, AssessmentRoot>(&host)
            .expect("the installed producer demand starts");
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            handle.settle(&request).unwrap()
        else {
            panic!("the bounded producer must settle");
        };
        assert_eq!(settled.producer_contacts_in_this_demand(), 1);
        assert!(settled.application_commit_receipt().is_some());
        let subject = settled
            .outputs_of::<RetentionAssessmentOutputs>()
            .unwrap()
            .entity::<AssessmentOutput>()
            .unwrap()
            .entity_id();
        let (checkpoint, sections) = runtime
            .capture_application_checkpoint_with_sections()
            .unwrap();
        assert_eq!(
            sections.accepted_output_count(),
            1,
            "the actual ready output is captured"
        );
        assert_captured_document_predicate(
            checkpoint.bytes(),
            sections.accepted_output_bytes(),
            subject,
        );
        (checkpoint, subject, settled)
    };
    let restored = restore(
        validated_program(),
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the producer and output are readmitted by their installed owners");
    let runtime = restored.runtime();
    let (recaptured, sections) = runtime
        .capture_application_checkpoint_with_sections()
        .unwrap();
    assert_eq!(sections.accepted_output_count(), 1);
    assert_captured_document_predicate(
        recaptured.bytes(),
        sections.accepted_output_bytes(),
        subject,
    );
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let mut handle = request
        .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .start_in_program::<_, AssessmentRoot>(&restored)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        handle.settle(&request).unwrap()
    else {
        panic!("the restored output must settle");
    };
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        0,
        "restored complete handler facts must authorize reuse without rerunning the producer"
    );
    assert!(
        settled.application_commit_receipt().is_none(),
        "this authority came from the checkpoint"
    );
    assert_eq!(
        settled
            .outputs_of::<RetentionAssessmentOutputs>()
            .unwrap()
            .entity::<AssessmentOutput>()
            .unwrap()
            .entity_id(),
        subject
    );
    let unchanged = request.retain_read().unwrap();
    assert!(matches!(request.at(&unchanged)
        .require_current_output_demand(&foreign_settlement, NonZeroUsize::new(4096).unwrap()),
        Err(WorthQueryOutputCurrentnessDenial::Output(e)) if e.kind()==WorthQueryOutputDemandDenialKind::ForeignSettlement));
    drop(foreign_settlement);
    request
        .at(&unchanged)
        .require_current_output_demand(&settled, NonZeroUsize::new(4096).unwrap())
        .expect("receipt-free restored output has current native lineage");
    assert!(matches!(request.at(&unchanged)
        .require_current_output_demand(&settled, NonZeroUsize::new(1).unwrap()),
        Err(WorthQueryOutputCurrentnessDenial::Output(e)) if e.kind()==WorthQueryOutputDemandDenialKind::WorkBudgetExceeded));
    drop(handle);

    assert_eq!(
        settle(set_retention(&restored, restored.current_world(), 6, 871)),
        RetentionVerdict::Performed(6)
    );
    let current = request.retain_read().unwrap();
    assert!(matches!(request.at(&current)
        .require_current_output_demand(&settled, NonZeroUsize::new(4096).unwrap()),
        Err(WorthQueryOutputCurrentnessDenial::Output(e)) if e.kind()==WorthQueryOutputDemandDenialKind::Superseded));
    drop(settled);
    let mut changed = request
        .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .start_in_program::<_, AssessmentRoot>(&restored)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(refreshed) =
        changed.settle(&request).unwrap()
    else {
        panic!("changed source must produce a fresh assessment");
    };
    assert_eq!(refreshed.producer_contacts_in_this_demand(), 1);
    assert!(refreshed.application_commit_receipt().is_some());
    let current = request.retain_read().unwrap();
    request
        .at(&current)
        .require_current_output_demand(&refreshed, NonZeroUsize::new(4096).unwrap())
        .expect("fresh producer settlement is current");
}

fn assert_captured_document_predicate(bytes: &[u8], accepted_bytes: usize, subject: EntityId) {
    // Inspect only Query's accepted-output section. The native payload remains opaque.
    // This independent v8 wire expectation fails if capture or readmission drops
    // the predicate while retaining the document's ordinary field observations.
    assert_eq!(&bytes[40..42], &8_u16.to_be_bytes());
    assert_eq!(DOCUMENT_IDENTITY, "document-1");
    let value = br#"{"String":{"Raw":"document-1"}}"#;
    let mut expected = Vec::new();
    expected.extend_from_slice(&(value.len() as u32).to_be_bytes());
    expected.extend_from_slice(value);
    expected.extend_from_slice(&2_u64.to_be_bytes());
    expected.extend_from_slice(&1_u32.to_be_bytes());
    expected.extend_from_slice(&subject.partition_value().to_be_bytes());
    expected.extend_from_slice(&subject.local_slot_value().to_be_bytes());
    expected.extend_from_slice(&subject.generation_value().to_be_bytes());
    let accepted = &bytes[bytes.len() - accepted_bytes..];
    assert_eq!(
        accepted
            .windows(expected.len())
            .filter(|window| *window == expected)
            .count(),
        1,
        "the complete document equality predicate must survive capture and readmission"
    );
}

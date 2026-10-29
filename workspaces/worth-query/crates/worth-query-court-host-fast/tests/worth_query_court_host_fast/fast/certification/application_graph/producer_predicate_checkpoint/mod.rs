pub(crate) use worth_query_worlds::facade::certification::application_graph::producer_predicate_checkpoint::*;

pub(crate) mod ordinary_source;

pub(crate) mod transition;

#[test]
fn captured_producer_predicate_reuses_after_reopen_and_refreshes_after_source_change() {
    let (checkpoint, subject, foreign_settlement, accepted_section) = {
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
            panic!("the installed producer must settle");
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
            .with_host_advancement(|host_phase| {
                runtime
                    .capture_application_checkpoint_with_sections(
                        &host_phase,
                        CapturePolicy::SystemAllocation,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
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
        let accepted_section = checkpoint.bytes()
            [checkpoint.bytes().len() - sections.accepted_output_bytes()..]
            .to_vec();
        let (repeated, repeated_sections) = runtime
            .with_host_advancement(|host_phase| {
                runtime
                    .capture_application_checkpoint_with_sections(
                        &host_phase,
                        CapturePolicy::SystemAllocation,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        assert_eq!(repeated_sections.accepted_output_count(), 1);
        assert_eq!(
            &repeated.bytes()[repeated.bytes().len() - repeated_sections.accepted_output_bytes()..],
            accepted_section.as_slice(),
            "the accepted native head preserves its complete roles and predicate on repeated capture"
        );
        (checkpoint, subject, settled, accepted_section)
    };
    let restored = restore(
        validated_program(),
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the producer and output are readmitted by their installed owners");
    let runtime = restored.runtime();
    let (recaptured, sections) = runtime
        .with_host_advancement(|host_phase| {
            runtime
                .capture_application_checkpoint_with_sections(
                    &host_phase,
                    CapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(sections.accepted_output_count(), 1);
    assert_captured_document_predicate(
        recaptured.bytes(),
        sections.accepted_output_bytes(),
        subject,
    );
    assert_eq!(
        &recaptured.bytes()[recaptured.bytes().len() - sections.accepted_output_bytes()..],
        accepted_section.as_slice(),
        "fresh native readmission preserves the complete accepted record"
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
    let unchanged = runtime
        .with_host_advancement(|host_phase| request.retain_read(&host_phase).unwrap())
        .expect("installed host call opens");
    assert!(matches!(request.at(&unchanged)
        .require_current_output_demand(&foreign_settlement),
        Err(WorthQueryOutputCurrentnessDenial::Output(e)) if e.kind()==WorthQueryOutputDemandDenialKind::ForeignSettlement));
    drop(foreign_settlement);
    request
        .at(&unchanged)
        .require_current_output_demand(&settled)
        .expect("receipt-free restored output has current native lineage");
    drop(handle);

    assert_eq!(
        settle(set_retention(&restored, restored.current_world(), 6, 871)),
        RetentionVerdict::Performed(6)
    );
    let current = runtime
        .with_host_advancement(|host_phase| request.retain_read(&host_phase).unwrap())
        .expect("installed host call opens");
    assert!(matches!(request.at(&current)
        .require_current_output_demand(&settled),
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
    let current = runtime
        .with_host_advancement(|host_phase| request.retain_read(&host_phase).unwrap())
        .expect("installed host call opens");
    request
        .at(&current)
        .require_current_output_demand(&refreshed)
        .expect("fresh producer settlement is current");
}

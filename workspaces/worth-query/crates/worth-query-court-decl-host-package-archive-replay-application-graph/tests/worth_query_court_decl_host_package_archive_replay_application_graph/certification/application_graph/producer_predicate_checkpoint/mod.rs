pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::producer_predicate_checkpoint::*;

#[test]
fn captured_producer_predicate_reuses_after_reopen_and_refreshes_after_source_change() {
    let directory = unique_directory("producer-predicate-reopen");
    let (directory, subject) = {
        let host = open_directory(
            directory,
            validated_program(),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .unwrap();
        let subject = {
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
            settled
                .outputs_of::<RetentionAssessmentOutputs>()
                .unwrap()
                .entity::<AssessmentOutput>()
                .unwrap()
                .entity_id()
        };
        let directory = host.checkpoint_and_close_for_reopen();
        (directory, subject)
    };
    let foreign_host = open_directory(
        unique_directory("producer-predicate-foreign"),
        validated_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap();
    let foreign_settlement = {
        let runtime = foreign_host.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(runtime.installed_schema(), &scope);
        let request = runtime.request(&principal, &scope);
        let mut handle = request
            .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
            .start_in_program::<_, AssessmentRoot>(&foreign_host)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            handle.settle(&request).unwrap()
        else {
            panic!("foreign producer must settle");
        };
        settled
    };
    let restored = open_directory(
        directory,
        validated_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the producer and output are readmitted by their installed owners");
    let runtime = restored.runtime();
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
        "this authority came from Directory Home recovery"
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

pub(crate) mod ordinary_source;

mod changed_build;

mod directory_changed_build;
mod directory_source_custody;

mod directory_discovered_source;

mod directory_source_supersession;

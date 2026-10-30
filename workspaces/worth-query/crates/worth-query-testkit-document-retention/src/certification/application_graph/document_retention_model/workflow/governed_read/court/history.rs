//! Genuine wait/proposal/assessment/approval stimuli and governed reference reacquisition.
pub use super::history_stimuli as stimuli;
pub use super::*;
pub use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
};
#[path = "history/pages.rs"]
pub mod pages;
pub fn target(
    application: &GovernedRuntime,
    basis: &WorthQueryApplicationReadObservation,
) -> WorthQueryWorkflowReadTarget {
    let runtime = application.runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let definition = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("reviewed-document").unwrap(),
    );
    let page = runtime
        .request(&operator, &scope)
        .at(basis)
        .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
            application.vocabulary(),
            &definition,
            input(),
            WorthQueryWorkflowReadExtent::Definition,
            1,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap();
    assert_eq!(page.entries().len(), 1);
    page.entries()[0]
        .target()
        .expect("real native indexed instance")
        .clone()
}
pub fn page(
    application: &GovernedRuntime,
    principal: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    scope: &WorthQueryRequestScope,
    basis: &WorthQueryApplicationReadObservation,
    target: &WorthQueryWorkflowReadTarget,
) -> WorthQueryWorkflowHistoryPage {
    application
        .runtime()
        .request(principal, scope)
        .at(basis)
        .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            target,
            input(),
            input(),
            16,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap()
}
pub fn start(application: &GovernedRuntime, key: u64) -> PublishedWorkflowInstanceRef {
    let definition = publication::publish_definition(
        application,
        super::super::super::super::workflow::governed_reviewed_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    );
    let prepared = prepare(application, definition, key + 1);
    let original = prepared.discovery_intent();
    let started = match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("genuine governed start: {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let basis = runtime
        .request(&principal, &scope)
        .retain_current_read()
        .unwrap();
    let found = read(application, &principal, &scope, &basis, &original).unwrap();
    assert!(found.target().is_some());
    // Only the instance ref leaves this helper; later locators need no start token.
    started.instance().clone()
}
pub fn propose(
    application: &GovernedRuntime,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> [u64; 3] {
    match stimuli::advance_instance_on(application, instance.branch(), instance.clone(), key)
        .unwrap()
    {
        WorkflowProgressOutcome::AwaitingProposal(required) => {
            assert_eq!(required.node_path(), "propose");
            assert!(!required.operation().is_empty());
            assert!(!required.input_type().is_empty());
        }
        other => panic!("existing native AwaitingProposal must own the need: {other:?}"),
    }
    match stimuli::propose_authoring_instance_with_retention(application, instance, key + 1, 8)
        .unwrap()
    {
        WorkflowProposalOutcome::Published(done) => {
            assert_eq!(done.proposal().node_path(), "propose");
            let principal = done.receipt().principal_scope().principal();
            let actor = [
                u64::from(principal.partition_id()),
                principal.local_slot(),
                u64::from(principal.generation()),
            ];
            drop(done);
            actor
        }
        other => panic!("genuine native proposal publication: {other:?}"),
    }
}

pub fn collect_to_approval(
    application: &GovernedRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) -> RequiredWorkflowApproval {
    for offset in [0, 10] {
        let settlement = stimuli::settle_assessment_for(
            application,
            instance.clone(),
            key + offset,
            DOCUMENT_IDENTITY,
        );
        assert!(matches!(
            stimuli::accept_assessment(
                application,
                instance.clone(),
                &settlement,
                key + offset + 1
            )
            .unwrap(),
            WorkflowProgressOutcome::Completed(_)
        ));
    }
    assert!(matches!(
        stimuli::advance_instance_on(application, instance.branch(), instance.clone(), key + 20)
            .unwrap(),
        WorkflowProgressOutcome::Completed(_)
    ));
    match stimuli::advance_instance_on(application, instance.branch(), instance.clone(), key + 21)
        .unwrap()
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("native joined evidence must require approval: {other:?}"),
    }
}

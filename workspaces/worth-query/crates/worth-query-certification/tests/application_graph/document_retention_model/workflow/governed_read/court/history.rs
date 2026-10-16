//! Genuine wait/proposal/assessment/approval stimuli and governed reference reacquisition.
use super::history_stimuli as stimuli;
use super::*;
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
};
#[path = "history/pages.rs"]
mod pages;
fn target(
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
fn page(
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
fn start(application: &GovernedRuntime, key: u64) -> PublishedWorkflowInstanceRef {
    let definition = publication::publish_definition(
        application,
        crate::document_retention_model::workflow::governed_reviewed_definition(),
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
fn propose(
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
#[test]
fn governed_history_reacquires_proposal_and_separately_audits_real_approval_at_exact_bases() {
    let (application, authentication) = super::super::runtime::retain_with_auth(
        crate::document_retention_model::host::publish_on_first_program(),
    );
    let instance = start(&application, 96_000);
    let expected_proposer = propose(&application, instance.clone(), 96_010);
    let runtime = application.runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let old = runtime
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    let old_target = target(&application, &old);
    let before = runtime.workflow_instance_progress_counters();
    let base = page(&application, &operator, &scope, &old, &old_target);
    assert_eq!(base.entries().len(), 1);
    assert!(base.entries()[0].approval().is_none());
    assert!(matches!(
        base.entries()[0].audit(),
        WorthQueryWorkflowHistoryAudit::Withheld
    ));
    let recovered = base.entries()[0]
        .proposal()
        .expect("native proposal locator reacquired after original ref drop")
        .clone();
    assert_eq!(recovered.instance_entity_id(), instance.entity_id());
    assert_eq!(recovered.node_path(), "propose");
    let audited = page(&application, &reviewer, &scope, &old, &old_target);
    let proposer = match audited.entries()[0].audit() {
        WorthQueryWorkflowHistoryAudit::Visible {
            proposer: Some(actor),
            approver: None,
            ..
        } => actor.entity_id(),
        _ => panic!("independent audit reveals proposer, not an absent approval"),
    };
    assert_eq!(
        runtime.workflow_instance_progress_counters(),
        before,
        "read spending cannot warm or evict command progression"
    );
    assert_eq!(
        [
            proposer.partition_value_u64(),
            proposer.local_slot_value(),
            u64::from(proposer.generation_value())
        ],
        expected_proposer
    );
    let mut subscription = runtime
        .request(&operator, &scope)
        .at(&old)
        .subscribe_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
            application.vocabulary(),
            &old_target,
            input(),
        )
        .unwrap();
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .at(&old)
            .poll_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input()
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Pending
    );
    let required = collect_to_approval(&application, &instance, 96_020);
    let expected_approver = match stimuli::approve_instance(
        &application,
        instance.clone(),
        &authentication,
        &required,
        &recovered,
        WorkflowApprovalDecision::Approve,
        96_042,
    )
    .unwrap()
    {
        WorkflowProgressOutcome::Completed(done) => done
            .approval()
            .expect("authentic performed approval")
            .approver(),
        other => panic!("genuine authenticated approval must commit: {other:?}"),
    };
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .at(&old)
            .poll_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input()
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Changed
    );
    // This Changed sequence covers workflow publications. Relation-only interest is proved at its native matcher.
    let now = runtime
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    let now_target = target(&application, &now);
    let before = runtime.workflow_instance_progress_counters();
    let fresh = page(&application, &reviewer, &scope, &now, &now_target);
    let approved = fresh
        .entries()
        .iter()
        .find(|row| row.approval().is_some())
        .expect("recorded real approval, not a synthesized result");
    assert_eq!(
        approved.approval().unwrap().decision(),
        WorkflowApprovalDecision::Approve
    );
    assert_eq!(
        approved.approval().unwrap().proposal_identity(),
        recovered.identity()
    );
    match approved.audit() {
        WorthQueryWorkflowHistoryAudit::Visible {
            approver: Some(actor),
            ..
        } => {
            assert_eq!(actor.entity_id(), expected_approver);
            assert_ne!(actor.entity_id(), proposer);
        }
        _ => panic!("actual independent approver must be audited"),
    };
    let historical = page(&application, &reviewer, &scope, &old, &old_target);
    assert_eq!(historical.basis().selected_commit(), old.selected_commit());
    assert_eq!(historical.entries().len(), 1);
    assert!(historical.entries()[0].approval().is_none());
    assert_eq!(runtime.workflow_instance_progress_counters(), before);
    let mut closing = runtime
        .request(&operator, &scope)
        .at(&now)
        .subscribe_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
            application.vocabulary(),
            &now_target,
            input(),
        )
        .unwrap();
    runtime.close_live_delivery();
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .at(&now)
            .poll_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut closing,
                input()
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Closed
    );
    closing.close();
    subscription.close();
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .retain_current_read()
            .unwrap()
            .selected_commit(),
        now.selected_commit(),
        "view/subscription close cannot publish workflow state"
    );
}

fn collect_to_approval(
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

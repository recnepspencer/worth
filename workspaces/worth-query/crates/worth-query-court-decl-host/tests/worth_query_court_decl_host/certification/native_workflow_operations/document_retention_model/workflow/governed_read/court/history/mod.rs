pub(crate) use worth_query_testkit_document_retention::facade::document_retention_model::workflow::governed_read::court::history::*;

#[test]
fn governed_history_reacquires_proposal_and_separately_audits_real_approval_at_exact_bases() {
    let (application, authentication) = super::super::runtime::retain_with_auth(
        worth_query_testkit_document_retention::facade::document_retention_model::host::publish_on_first_program(),
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

pub(crate) mod pages;

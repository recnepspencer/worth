//! Later requests disclose exact history; live marks never refresh retained data.
use crate::{
    definition::MixedWorkflow,
    document_retention_model::{
        operator_identity::{authenticate_operator, authenticate_reviewer, request_scope},
        retention_entry::DOCUMENT_IDENTITY,
        schema::DocumentRetentionSchema,
        workflow::governed_read::{AuditRead, SubjectRead, WorkflowReadInput},
        MixedApplication,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_discovery::*,
    application_entry::*,
};

/// The closure performs native approval using its original need and fresh authentication.
/// Disclosed history supplies no approval authority.
pub fn observe_approval(
    application: &MixedApplication,
    instance: &PublishedWorkflowInstanceRef,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>,
    proposal: &PublishedWorkflowProposalRef,
    expected_proposer: [u64; 3],
    approve: impl FnOnce() -> [u64; 3],
) -> [u64; 3] {
    let runtime = application.runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let old = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .retain_current_read()
        .expect("retain the pre-approval basis");
    let discovered = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .at(&old)
        .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original,
            input(),
            input(),
        )
        .expect("later-request original-start disclosure");
    let target = discovered
        .target()
        .expect("the real original instance")
        .clone();
    let withheld = page(application, &operator, &scope, &old, &target);
    let proposal_row = withheld
        .entries()
        .iter()
        .find(|row| row.proposal().is_some())
        .expect("canonical proposal remains visible without audit");
    assert_eq!(proposal_row.proposal().unwrap(), proposal);
    assert!(withheld
        .entries()
        .iter()
        .all(|row| row.approval().is_none()));
    assert!(withheld
        .entries()
        .iter()
        .all(|row| matches!(row.audit(), WorthQueryWorkflowHistoryAudit::Withheld)));
    let audited = page(application, &reviewer, &scope, &old, &target);
    let proposal_row = audited
        .entries()
        .iter()
        .find(|row| row.proposal().is_some())
        .unwrap();
    let proposer = match proposal_row.audit() {
        WorthQueryWorkflowHistoryAudit::Visible {
            proposer: Some(actor),
            ..
        } => actor.entity_id(),
        _ => panic!("independently admitted audit must reveal the canonical proposer"),
    };
    assert_eq!(
        [
            proposer.partition_value_u64(),
            proposer.local_slot_value(),
            u64::from(proposer.generation_value()),
        ],
        expected_proposer,
        "the original native proposal receipt is the actor oracle"
    );

    // compiled-example: begin
    let mut subscription = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .at(&old)
        .subscribe_workflow_history::<MixedWorkflow, SubjectRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .expect("native envelope interest at the old data basis");
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .on_branch(instance.branch())
            .at(&old)
            .poll_workflow_history::<MixedWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input(),
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Pending
    );
    let expected_approver = approve();
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .on_branch(instance.branch())
            .at(&old)
            .poll_workflow_history::<MixedWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input(),
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Changed
    );
    // A mark reports relevant change. It does not mutate the old retained read.
    let historical = page(application, &reviewer, &scope, &old, &target);
    assert_eq!(historical.basis().selected_commit(), old.selected_commit());
    assert!(historical
        .entries()
        .iter()
        .all(|row| row.approval().is_none()));
    // Refresh is an explicit caller choice with new current admission.
    let now = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .retain_current_read()
        .expect("explicitly retain a fresh post-approval basis");
    let fresh_discovery = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .at(&now)
        .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original,
            input(),
            input(),
        )
        .unwrap();
    let fresh_target = fresh_discovery
        .target()
        .expect("same native instance at fresh data");
    let fresh = page(application, &reviewer, &scope, &now, fresh_target);
    assert_eq!(fresh.basis().selected_commit(), now.selected_commit());
    let approved = fresh
        .entries()
        .iter()
        .find(|row| row.approval().is_some())
        .expect("actual recorded approval");
    assert_eq!(
        approved.approval().unwrap().decision(),
        WorkflowApprovalDecision::Approve
    );
    assert_eq!(
        approved.approval().unwrap().proposal_identity(),
        proposal.identity()
    );
    match approved.audit() {
        WorthQueryWorkflowHistoryAudit::Visible {
            approver: Some(actor),
            ..
        } => {
            let actor = actor.entity_id();
            assert_eq!(
                [
                    actor.partition_value_u64(),
                    actor.local_slot_value(),
                    u64::from(actor.generation_value())
                ],
                expected_approver,
                "the performed native approval is the actor oracle"
            );
            assert_ne!(actor, proposer);
        }
        _ => panic!("independent audit reveals the performed approver"),
    }
    let fresh_withheld = page(application, &operator, &scope, &now, fresh_target);
    assert_eq!(fresh.entries().len(), fresh_withheld.entries().len());
    for (audited, withheld) in fresh.entries().iter().zip(fresh_withheld.entries()) {
        assert_eq!(audited.occurrence(), withheld.occurrence());
        assert_eq!(audited.node_path(), withheld.node_path());
        assert_eq!(audited.outcome(), withheld.outcome());
        assert_eq!(audited.proposal(), withheld.proposal());
    }
    let approved_without_audit = fresh_withheld
        .entries()
        .iter()
        .find(|row| row.approval().is_some())
        .expect("withholding cannot hide approval state");
    assert!(matches!(
        approved_without_audit.audit(),
        WorthQueryWorkflowHistoryAudit::Withheld
    ));
    // Releases native subscription custody only; never operation cleanup proof.
    subscription.close();
    // compiled-example: end
    expected_approver
}

pub(crate) fn page(
    application: &MixedApplication,
    principal: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    scope: &WorthQueryRequestScope,
    basis: &WorthQueryApplicationReadObservation,
    target: &WorthQueryWorkflowReadTarget,
) -> WorthQueryWorkflowHistoryPage {
    let page = application
        .runtime()
        .request(principal, scope)
        .at(basis)
        .read_workflow_history::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            target,
            input(),
            input(),
            16,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .expect("fresh subject and independent audit admission for this exact page");
    assert!(
        page.continuation().is_none(),
        "this bounded workflow fits one complete page"
    );
    page
}
fn input() -> WorkflowReadInput {
    WorkflowReadInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}

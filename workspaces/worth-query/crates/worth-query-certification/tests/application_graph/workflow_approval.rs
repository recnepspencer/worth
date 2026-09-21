//! Certification of admitted workflow approval decisions and exact replay.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
    RequiredWorkflowApproval, WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorkflowTransitionPreparationDenial,
    WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitOutcome,
};

use super::bounded_dimension_model::{
    host::{BoundedDimensionWorkflowRuntime, SEED_DIMENSION},
    presented_request::set_dimension,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        accept_assessment, advance_instance, approve_instance, propose_instance,
        publish_definition, reviewed_geometry_definition, settle_assessment, start_instance,
    },
};

#[test]
fn approval_rejects_assessment_evidence_after_native_source_aba() {
    let (application, _, instance, proposal, required, _) = approval_journey("approved", 700);
    for (dimension, key) in [(SEED_DIMENSION + 1, 710), (SEED_DIMENSION, 711)] {
        assert_eq!(
            settle(set_dimension(
                application.program_runtime(),
                instance.branch(),
                dimension,
                key,
            )),
            DimensionVerdict::Performed(dimension)
        );
    }
    assert!(matches!(
        approve_instance(
            &application,
            instance,
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
            712,
        ),
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt)
        )) if attempt.kind()
            == WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
    ));
}

#[test]
fn exact_approval_persists_replays_and_binds_decision_and_instance() {
    let (application, definition, instance, proposal, required, mut expected_evidence) =
        approval_journey("approved", 500);
    expected_evidence.sort_unstable();
    let committed_approval = match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        510,
    )
    .expect("the exact approval request must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "approval");
            assert!(!performed.replayed());
            let approval = performed
                .approval()
                .expect("approval publication must reconstruct its durable approval fact");
            assert_ne!(approval.entity(), performed.transition());
            assert_eq!(approval.identity().len(), 64);
            assert_eq!(approval.decision(), WorkflowApprovalDecision::Approve);
            assert_eq!(approval.proposal(), proposal.entity_id());
            let mut actual_evidence = approval.evidence().to_vec();
            actual_evidence.sort_unstable();
            assert_eq!(actual_evidence, expected_evidence);
            assert_eq!(approval.purpose(), "String(Raw(\"reviewed-geometry\"))");
            assert_eq!(approval.expiry(), u64::MAX);
            let principal = performed.receipt().principal_scope().principal();
            assert_eq!(
                approval.approver(),
                worth_relational::facade::identity::EntityId::new(
                    worth_relational::facade::identity::PartitionId::new(principal.partition_id(),),
                    principal.local_slot(),
                    principal.generation(),
                )
            );
            (approval.entity(), approval.identity().to_owned())
        }
        other => panic!("expected a performed workflow approval, got {other:?}"),
    };
    match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        510,
    )
    .expect("the exact approval retry must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert!(performed.replayed());
            let approval = performed
                .approval()
                .expect("approval replay must reconstruct the durable approval fact");
            let (entity, identity) = &committed_approval;
            let mut actual_evidence = approval.evidence().to_vec();
            actual_evidence.sort_unstable();
            assert_eq!(approval.entity(), *entity);
            assert_eq!(approval.identity(), identity);
            assert_eq!(approval.decision(), WorkflowApprovalDecision::Approve);
            assert_eq!(approval.proposal(), proposal.entity_id());
            assert_eq!(actual_evidence, expected_evidence);
            let principal = performed.receipt().principal_scope().principal();
            assert_eq!(
                approval.approver(),
                worth_relational::facade::identity::EntityId::new(
                    worth_relational::facade::identity::PartitionId::new(principal.partition_id(),),
                    principal.local_slot(),
                    principal.generation(),
                )
            );
            assert_eq!(approval.purpose(), "String(Raw(\"reviewed-geometry\"))");
            assert_eq!(approval.expiry(), u64::MAX);
        }
        other => panic!("expected a replayed workflow approval, got {other:?}"),
    }
    match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Reject,
        510,
    )
    .expect("changed approval meaning must resolve through idempotency")
    {
        WorkflowProgressOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
            denial,
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
        ),
        other => panic!("expected approval decision intent drift, got {other:?}"),
    }

    let foreign_definition = match publish_definition(
        &application,
        reviewed_geometry_definition("foreign"),
        WorkflowDefinitionExpectedPredecessor::Published(definition),
        512,
    )
    .expect("foreign definition publication must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected a foreign definition publication, got {other:?}"),
    };
    let foreign = match start_instance(&application, foreign_definition.definition().clone(), 513)
        .expect("foreign instance start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected a foreign instance, got {other:?}"),
    };
    let foreign_proposal = published_proposal(&application, foreign.clone(), 514);
    for denial in [
        approve_instance(
            &application,
            instance.clone(),
            &required,
            &foreign_proposal,
            WorkflowApprovalDecision::Approve,
            515,
        ),
        approve_instance(
            &application,
            foreign,
            &required,
            &foreign_proposal,
            WorkflowApprovalDecision::Approve,
            516,
        ),
    ] {
        assert!(matches!(
            denial,
            Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
                WorkflowTransitionPreparationDenial::Attempt(attempt)
            )) if attempt.kind()
                == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch
        ));
    }
}

#[test]
fn rejection_routes_to_its_declared_terminal_and_replays_before_that_successor() {
    let (application, _, instance, proposal, required, _) = approval_journey("unused", 600);
    for replayed in [false, true] {
        match approve_instance(
            &application,
            instance.clone(),
            &required,
            &proposal,
            WorkflowApprovalDecision::Reject,
            610,
        )
        .expect("rejection and its exact retry must prepare")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert_eq!(performed.node_path(), "approval");
                assert_eq!(performed.replayed(), replayed);
            }
            other => panic!("expected a performed rejection, got {other:?}"),
        }
    }
    match advance_instance(&application, instance, 611)
        .expect("the rejected successor terminal must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "rejected")
        }
        other => panic!("expected the rejected terminal, got {other:?}"),
    }
}

fn approval_journey(
    completion: &str,
    key: u64,
) -> (
    BoundedDimensionWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
    PublishedWorkflowProposalRef,
    RequiredWorkflowApproval,
    Vec<worth_relational::facade::identity::EntityId>,
) {
    let application = super::bounded_dimension_model::host::publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition(completion),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("approval definition publication must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected a published approval definition, got {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), key + 1)
        .expect("approval instance start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected a started approval instance, got {other:?}"),
    };
    let proposal = published_proposal(&application, instance.clone(), key + 2);
    let mut evidence = Vec::new();
    for offset in [3, 5] {
        let settlement = settle_assessment(&application, instance.clone(), key + offset);
        match accept_assessment(
            &application,
            instance.clone(),
            &settlement,
            key + offset + 1,
        ) {
            Ok(WorkflowProgressOutcome::Completed(performed)) => evidence.push(
                performed
                    .assessment_evidence()
                    .expect("accepted assessment must expose its evidence entity")
                    .evidence(),
            ),
            other => panic!("expected accepted assessment, got {other:?}"),
        }
    }
    match advance_instance(&application, instance.clone(), key + 7) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected completed evidence join, got {other:?}"),
    }
    let required = match advance_instance(&application, instance.clone(), key + 8)
        .expect("approval requirement must prepare")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected an approval requirement, got {other:?}"),
    };
    (
        application,
        definition.definition().clone(),
        instance,
        proposal,
        required,
        evidence,
    )
}

fn published_proposal(
    application: &BoundedDimensionWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> PublishedWorkflowProposalRef {
    match propose_instance(application, instance, key).expect("proposal must prepare") {
        WorkflowProposalOutcome::Published(performed) => performed.proposal().clone(),
        other => panic!("expected a published proposal, got {other:?}"),
    }
}

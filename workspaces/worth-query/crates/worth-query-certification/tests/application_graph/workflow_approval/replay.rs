use super::*;

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
                    worth_relational::facade::identity::PartitionId::new(principal.partition_id()),
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
                    worth_relational::facade::identity::PartitionId::new(principal.partition_id()),
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

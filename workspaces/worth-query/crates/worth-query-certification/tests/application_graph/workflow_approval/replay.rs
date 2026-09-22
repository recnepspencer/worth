use super::*;

#[test]
fn older_approval_replays_after_the_same_node_settles_again() {
    let application =
        super::super::bounded_dimension_model::host::publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        super::super::bounded_dimension_model::workflow::approval_retry_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        950,
    )
    .expect("retry definition publishes")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("expected publication, got {other:?}"),
    };
    let instance = match start_instance(&application, definition, 951).expect("instance starts") {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected start, got {other:?}"),
    };
    let proposal = super::proposal::published_proposal(&application, instance.clone(), 952);
    for key in [953, 955] {
        let settlement = settle_assessment(&application, instance.clone(), key);
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &settlement, key + 1),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    assert!(matches!(
        advance_instance(&application, instance.clone(), 957),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let mut first = None;
    for key in [958, 960] {
        let required = match advance_instance(&application, instance.clone(), key)
            .expect("approval selected")
        {
            WorkflowProgressOutcome::AwaitingApproval(required) => required,
            other => panic!("expected approval, got {other:?}"),
        };
        let before = application.runtime().workflow_instance_progress_counters();
        let transition = match approve_instance(
            &application,
            instance.clone(),
            &required,
            &proposal,
            WorkflowApprovalDecision::Reject,
            key + 1,
        )
        .expect("rejection prepares")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert!(!performed.replayed());
                performed.transition()
            }
            other => panic!("expected rejection, got {other:?}"),
        };
        let after = application.runtime().workflow_instance_progress_counters();
        assert_eq!(
            after.warm_history_transition_visits(),
            before.warm_history_transition_visits()
        );
        if first.is_none() {
            first = Some((required, transition));
        }
    }
    let (required, transition) = first.unwrap();
    for cold in [false, true] {
        if cold {
            application
                .runtime()
                .release_workflow_instance_progress_for_test();
        }
        match approve_instance(
            &application,
            instance.clone(),
            &required,
            &proposal,
            WorkflowApprovalDecision::Reject,
            959,
        )
        .expect("older retry prepares")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert!(performed.replayed());
                assert_eq!(performed.transition(), transition);
            }
            other => panic!("expected original rejection replay, got {other:?}"),
        }
    }
}

#[test]
fn exact_approval_persists_replays_and_binds_decision_and_instance() {
    let (application, definition, instance, proposal, required, mut expected_evidence) =
        approval_journey("approved", 500);
    expected_evidence.sort_unstable();
    let before_approval = application.runtime().workflow_instance_progress_counters();
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
    let after_approval = application.runtime().workflow_instance_progress_counters();
    assert_eq!(
        after_approval.warm_core_hits(),
        before_approval.warm_core_hits() + 1
    );
    assert_eq!(
        after_approval.warm_history_transition_visits(),
        before_approval.warm_history_transition_visits()
    );
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let before_reconstruction = application.runtime().workflow_instance_progress_counters();
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
    let after_reconstruction = application.runtime().workflow_instance_progress_counters();
    assert_eq!(
        after_reconstruction.cold_misses(),
        before_reconstruction.cold_misses() + 1
    );
    assert!(
        after_reconstruction.cold_reconstruction_transition_visits()
            > before_reconstruction.cold_reconstruction_transition_visits()
    );
    assert_eq!(
        after_reconstruction.warm_history_transition_visits(),
        before_reconstruction.warm_history_transition_visits()
    );
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
    let foreign_proposal = super::proposal::published_proposal(&application, foreign.clone(), 514);
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

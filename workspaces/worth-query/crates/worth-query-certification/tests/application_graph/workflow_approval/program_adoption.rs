//! Approval custody across program adoption. Evidence collected under the
//! source program never authorizes a decision under the target, and an
//! approval whose guarded operation has not run blocks the move entirely.

use worth_query_host::facade::application_entry::{
    WorkflowInstancePreparationDenial, WorkflowProposalPreparationDenial,
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryWorkflowInstanceStartPreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryBranchAdoptionPreparationDenial, WorthQueryWorkflowAdoptionInventory,
    WorthQueryWorkflowDefinitionDisposition, WorthQueryWorkflowDispositionDenial,
    WorthQueryWorkflowInstanceCustody, WorthQueryWorkflowInstanceDisposition,
};

use super::super::bounded_dimension_model::{
    programs::DimensionProgramP1,
    workflow::{
        advance_on_second, approve_on_second, cancel_on_second, prepare_second_program_adoption,
        propose_on_second, publish_adoption, recollect_on_second,
        second_program_workflow_inventory, support_workflow_program,
    },
};
use super::fork_continuation::fork_of;
use super::instance_migration::perform_approved_effect;
use super::journey::approval_requirement;
use super::*;

#[test]
fn a_carried_instance_refuses_evidence_collected_under_the_source_program() {
    let (mut application, _, instance, proposal, _, _) = approval_journey("approved", 86_000);
    support_workflow_program::<DimensionProgramP1>(&mut application);
    let main = application.current_world();
    let inventory = second_program_workflow_inventory(&application, main);
    let waiting = inventory
        .instance(instance.entity_id())
        .expect("the instance waiting for approval is inventoried");
    assert!(!waiting.requires_migration(), "{waiting:?}");
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));

    let required = match advance_on_second(&application, main, instance.clone(), 86_010)
        .expect("the carried approval requirement prepares under P1")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected the carried instance to await approval, got {other:?}"),
    };
    assert!(matches!(
        approve_on_second(
            &application,
            instance.clone(),
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
            86_011,
        ),
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt)
        )) if attempt.kind()
            == WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
    ));

    for (node, key) in [
        ("checks/structural", 86_020),
        ("checks/manufacturability", 86_030),
    ] {
        match recollect_on_second(&application, instance.clone(), node, key) {
            Ok(WorkflowProgressOutcome::Completed(_)) => {}
            other => panic!("{node} did not collect fresh P1 evidence: {other:?}"),
        }
    }
    // The requirement names its evidence; fresh evidence means a fresh one.
    let required = match advance_on_second(&application, main, instance.clone(), 86_039)
        .expect("the approval requirement re-reads fresh P1 evidence")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected the recollected instance to await approval, got {other:?}"),
    };
    match approve_on_second(
        &application,
        instance,
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        86_040,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("fresh P1 evidence authorizes the approval, got {other:?}"),
    }
}

#[test]
fn a_sibling_instance_keeps_its_program_after_its_parent_adopts() {
    let (mut application, definition, _, _, _, _) = approval_journey("approved", 86_300);
    support_workflow_program::<DimensionProgramP1>(&mut application);
    let main = application.current_world();
    let sibling = fork_of(&application, main);
    let unaffected = match start_instance(&application, definition.held_on(sibling), 86_310)
        .expect("the sibling starts under its own copy of the definition")
    {
        WorkflowInstanceStartOutcome::Started(started) => started.instance().clone(),
        other => panic!("expected a sibling instance, got {other:?}"),
    };
    // The sibling waits for approval while its parent moves to P1.
    let (proposal, required, _) = approval_requirement(&application, unaffected.clone(), 86_320);
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));

    // The carriage belongs to main's truth; the sibling's copy of the
    // definition still executes under P0, and its approved effect runs.
    match approve_instance(
        &application,
        unaffected.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        86_330,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("the sibling approves under P0, got {other:?}"),
    }
    perform_approved_effect(&application, &unaffected, 86_340);
}

#[test]
fn an_outstanding_approval_requires_migration_before_adoption() {
    let (mut application, _, instance, proposal, required, _) =
        approval_journey("approved", 86_100);
    match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        86_110,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected approval to complete, got {other:?}"),
    }
    support_workflow_program::<DimensionProgramP1>(&mut application);
    let main = application.current_world();
    let inventory = second_program_workflow_inventory(&application, main);
    let approved = inventory
        .instance(instance.entity_id())
        .expect("the approved instance is inventoried");
    assert_eq!(
        approved.custody(),
        &WorthQueryWorkflowInstanceCustody::ApprovalOutstanding {
            approval_node_path: "approval".to_owned(),
        },
    );
    assert!(approved.requires_migration());
    assert!(approved.legal_dispositions().is_empty());
    let migration_required = WorthQueryWorkflowDispositionDenial::MigrationRequired {
        instance: instance.entity_id(),
    };
    assert_eq!(
        inventory.carry_compatible(),
        Err(migration_required.clone())
    );
    assert_eq!(
        inventory
            .dispositions()
            .instance(approved, WorthQueryWorkflowInstanceDisposition::Cancel),
        Err(migration_required.clone()),
        "an approved obligation is never cancelled away",
    );

    let definitions_only = |inventory: &WorthQueryWorkflowAdoptionInventory| {
        inventory
            .dispositions()
            .definition(
                &inventory.definitions()[0],
                WorthQueryWorkflowDefinitionDisposition::Carry,
            )
            .unwrap()
    };
    match prepare_second_program_adoption(&application, main, Some(&definitions_only)) {
        Err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected(denial),
        )) => assert_eq!(denial, migration_required),
        Err(other) => panic!("the outstanding approval must be named: {other:?}"),
        Ok(_) => panic!("an outstanding approval must not be carried or dropped"),
    }
}

#[test]
fn a_cancelled_instance_names_its_cancellation_to_every_request() {
    let (mut application, _, instance, proposal, required, _) =
        approval_journey("approved", 86_200);
    support_workflow_program::<DimensionProgramP1>(&mut application);
    let main = application.current_world();
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| {
            let waiting = inventory
                .instance(instance.entity_id())
                .expect("the waiting instance is inventoried");
            inventory
                .dispositions()
                .definition(
                    &inventory.definitions()[0],
                    WorthQueryWorkflowDefinitionDisposition::Carry,
                )
                .and_then(|choices| {
                    choices.instance(waiting, WorthQueryWorkflowInstanceDisposition::Cancel)
                })
                .expect("an unperformed instance may be cancelled")
        }),
    ));

    let cancelled = |denial: &WorthQueryWorkflowAdvancePreparationDenial| {
        matches!(
            denial,
            WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
                WorkflowTransitionPreparationDenial::Attempt(attempt)
            ) if attempt.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled
        )
    };
    let advance = advance_on_second(&application, main, instance.clone(), 86_210)
        .expect_err("a cancelled instance never advances");
    assert!(cancelled(&advance), "{advance:?}");
    let decision = approve_on_second(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        86_211,
    )
    .expect_err("a cancelled instance takes no decision");
    assert!(cancelled(&decision), "{decision:?}");
    let proposal = propose_on_second(&application, instance.clone(), 86_212)
        .expect_err("a cancelled instance takes no proposal");
    assert!(
        matches!(
            &proposal,
            WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
                WorkflowProposalPreparationDenial::Attempt(attempt)
            ) if attempt.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled
        ),
        "{proposal:?}"
    );
    let cancellation = cancel_on_second(&application, instance, 86_213)
        .expect_err("an instance adoption cancelled takes no cancellation");
    assert!(
        matches!(
            &cancellation,
            WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
                WorkflowInstancePreparationDenial::Attempt(attempt)
            ) if attempt.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled
        ),
        "{cancellation:?}"
    );
}

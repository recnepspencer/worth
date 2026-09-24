//! Certification of immutable workflow proposals and retained replay.

use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    },
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome},
};

use super::bounded_dimension_model::{
    host::publish_workflow_on_first_program,
    workflow::{
        advance_instance, proposal_terminal_definition, propose_authoring_instance,
        propose_instance, propose_instance_on_branch, publish_definition,
        repeated_proposal_definition, reviewed_geometry_definition, start_instance,
    },
};

#[test]
fn public_real_operation_request_publishes_and_replays_one_immutable_proposal() {
    let application = publish_workflow_on_first_program();
    let definition = expect_published(publish_definition(
        &application,
        reviewed_geometry_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        401,
    ));
    let started = expect_started(start_instance(
        &application,
        definition.definition().clone(),
        402,
    ));
    let proposal = expect_proposal(propose_instance(
        &application,
        started.instance().clone(),
        403,
    ));
    assert_eq!(proposal.node_path(), "propose");
    assert!(!proposal.replayed());

    let replay = expect_proposal(propose_instance(
        &application,
        started.instance().clone(),
        403,
    ));
    assert!(replay.replayed());
    assert_eq!(replay.transition(), proposal.transition());
    assert_eq!(replay.proposal(), proposal.proposal());

    match propose_instance(&application, started.instance().clone(), 404)
        .expect("a new proposal key must reach retained-head comparison")
    {
        WorkflowProposalOutcome::Application(WorthQueryApplicationCommitOutcome::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0);
        }
        other => panic!("expected a stale duplicate proposal, got {other:?}"),
    }

    let other_instance = expect_started(start_instance(
        &application,
        definition.definition().clone(),
        405,
    ));
    match propose_instance(&application, other_instance.instance().clone(), 403)
        .expect("same-key proposal drift must reach idempotency comparison")
    {
        WorkflowProposalOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
            denial,
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
        ),
        other => panic!("expected proposal intent drift, got {other:?}"),
    }
}

#[test]
fn proposal_replay_is_key_bound_across_an_eligible_repeated_proposal_head() {
    let application = publish_workflow_on_first_program();
    let definition = expect_published(publish_definition(
        &application,
        repeated_proposal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        411,
    ));
    let started = expect_started(start_instance(
        &application,
        definition.definition().clone(),
        412,
    ));
    let first = expect_proposal(propose_authoring_instance(
        &application,
        started.instance().clone(),
        413,
    ));
    assert_eq!(first.node_path(), "proposal/first");

    let replay = expect_proposal(propose_authoring_instance(
        &application,
        started.instance().clone(),
        413,
    ));
    assert!(replay.replayed());
    assert_eq!(replay.proposal(), first.proposal());

    let second = expect_proposal(propose_authoring_instance(
        &application,
        started.instance().clone(),
        414,
    ));
    assert!(!second.replayed());
    assert_eq!(second.node_path(), "proposal/second");
    assert_ne!(second.proposal(), first.proposal());
}

#[test]
fn proposal_replay_resolves_after_the_instance_completed() {
    let application = publish_workflow_on_first_program();
    let definition = expect_published(publish_definition(
        &application,
        proposal_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        421,
    ));
    let started = expect_started(start_instance(
        &application,
        definition.definition().clone(),
        422,
    ));
    let proposal = expect_proposal(propose_authoring_instance(
        &application,
        started.instance().clone(),
        423,
    ));
    match advance_instance(&application, started.instance().clone(), 424)
        .expect("terminal completion must prepare")
    {
        WorkflowProgressOutcome::Completed(_) => {}
        other => panic!("expected terminal completion, got {other:?}"),
    }

    let replay = expect_proposal(propose_authoring_instance(
        &application,
        started.instance().clone(),
        423,
    ));
    assert!(replay.replayed());
    assert_eq!(replay.proposal(), proposal.proposal());
    assert_eq!(replay.transition(), proposal.transition());
}

#[test]
fn proposal_replay_rejects_a_foreign_branch_instance_reference_before_lookup() {
    let application = publish_workflow_on_first_program();
    let definition = expect_published(publish_definition(
        &application,
        proposal_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        431,
    ));
    let started = expect_started(start_instance(
        &application,
        definition.definition().clone(),
        432,
    ));
    expect_proposal(propose_authoring_instance(
        &application,
        started.instance().clone(),
        433,
    ));
    let main = application.runtime().current_world();
    let sibling = application
        .runtime()
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the sibling branch publishes");

    let denial = propose_instance_on_branch(&application, sibling, started.instance().clone(), 433)
        .expect_err("a foreign-branch instance reference must be denied before replay lookup");
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::application_entry::WorthQueryWorkflowProposalPreparationDenialKind::InstanceBranchMismatch
    );
}

fn expect_published(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        worth_query_host::facade::application_entry::WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) -> worth_query_host::facade::application_entry::PerformedWorkflowDefinitionPublication {
    match result.expect("workflow definition publication must prepare") {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected a published workflow definition, got {other:?}"),
    }
}

fn expect_started(
    result: Result<
        WorkflowInstanceStartOutcome,
        worth_query_host::facade::application_entry::WorthQueryWorkflowInstanceStartPreparationDenial,
    >,
) -> worth_query_host::facade::application_entry::PerformedWorkflowInstanceStart {
    match result.expect("workflow instance start must prepare") {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected a started workflow instance, got {other:?}"),
    }
}

fn expect_proposal(
    result: Result<
        WorkflowProposalOutcome,
        worth_query_host::facade::application_entry::WorthQueryWorkflowProposalPreparationDenial,
    >,
) -> worth_query_host::facade::application_entry::PerformedWorkflowProposal {
    match result.expect("workflow proposal preparation must succeed") {
        WorkflowProposalOutcome::Published(proposal) => proposal,
        other => panic!("expected a published workflow proposal, got {other:?}"),
    }
}

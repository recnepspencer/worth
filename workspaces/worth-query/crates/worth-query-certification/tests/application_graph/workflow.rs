//! Public certification for authored workflow-definition publication.

use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
        WorthQueryWorkflowInstanceStartPreparationDenial,
    },
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome},
};

use super::bounded_dimension_model::{
    host::publish_workflow_on_first_program,
    workflow::{
        advance_instance, publish_definition, reviewed_geometry_definition, start_instance,
        terminal_definition,
    },
};

#[test]
fn installed_resource_ceiling_rejects_a_definition_declaring_broader_limits() {
    let resources =
        worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling::new(
            32,
            64,
            1,
            ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
            64 * 1024,
            32,
            128,
            256 * 1024,
        )
        .expect("the constrained workflow resources are nonzero");
    let application = super::bounded_dimension_model::workflow::retain_workflow_with_resources(
        super::bounded_dimension_model::host::publish_on_first_program(),
        resources,
    );
    let denial = match application.workflow_spec().bind_definition(
        super::bounded_dimension_model::workflow::reviewed_geometry_definition("completed"),
    ) {
        Ok(_) => panic!("definition limits exceeded the installed effect ceiling"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        worth_query_installation::facade::WorthQueryApplicationWorkflowInstallationDenialKind::DefinitionLimitExceeded
    );
}

#[test]
fn installed_component_ceiling_rejects_only_the_broader_component_contract() {
    let resources =
        worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling::new(
            32,
            64,
            4,
            ApplicationWorkflowComponentLimits::new(31, 4, 128, 256, 256).unwrap(),
            64 * 1024,
            32,
            128,
            256 * 1024,
        )
        .expect("the constrained workflow resources are nonzero");
    let application = super::bounded_dimension_model::workflow::retain_workflow_with_resources(
        super::bounded_dimension_model::host::publish_on_first_program(),
        resources,
    );
    let denial = match application.workflow_spec().bind_definition(
        super::bounded_dimension_model::workflow::reviewed_geometry_definition("completed"),
    ) {
        Ok(_) => panic!("definition component limits exceed installed resources"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        worth_query_installation::facade::WorthQueryApplicationWorkflowInstallationDenialKind::DefinitionLimitExceeded
    );
}

#[test]
fn public_terminal_workflow_advances_once_through_authenticated_transition_authority() {
    let application = publish_workflow_on_first_program();
    let before = application.runtime().workflow_compilation_reuse_counters();
    let definition = expect_published(
        "terminal definition",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            31,
        ),
    );
    let started = expect_started(start_instance(
        &application,
        definition.definition().clone(),
        32,
    ));
    let after_cold_start = application.runtime().workflow_compilation_reuse_counters();
    assert_eq!(after_cold_start.cold_misses(), before.cold_misses() + 1);
    assert_eq!(after_cold_start.cold_retains(), before.cold_retains() + 1);
    assert_eq!(started.instance().current_node_path(), "completed");
    for key in 100..131 {
        expect_started(start_instance(
            &application,
            definition.definition().clone(),
            key,
        ));
    }
    let after_warm_starts = application.runtime().workflow_compilation_reuse_counters();
    assert_eq!(
        after_warm_starts.cold_misses(),
        after_cold_start.cold_misses()
    );
    assert_eq!(
        after_warm_starts.warm_hits(),
        after_cold_start.warm_hits() + 31
    );
    expect_stale_start(start_instance(
        &application,
        definition.definition().clone(),
        131,
    ));
    let transition = match advance_instance(&application, started.instance().clone(), 33)
        .expect("workflow advance preparation must succeed")
    {
        WorkflowProgressOutcome::Completed(transition) => transition,
        other => panic!("expected a completed terminal transition, got {other:?}"),
    };
    assert_eq!(transition.node_path(), "completed");
    assert!(transition.terminal());
    assert!(!transition.replayed());
    let replay = match advance_instance(&application, started.instance().clone(), 33)
        .expect("an exact workflow transition retry must prepare")
    {
        WorkflowProgressOutcome::Completed(transition) => transition,
        other => panic!("expected a replayed terminal transition, got {other:?}"),
    };
    assert!(replay.replayed());
    assert!(replay.terminal());
    assert_eq!(replay.transition(), transition.transition());
    match advance_instance(&application, started.instance().clone(), 34)
        .expect("a new key must reach transition-head comparison")
    {
        WorkflowProgressOutcome::Application(WorthQueryApplicationCommitOutcome::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0)
        }
        other => panic!("expected a stale second terminal occurrence, got {other:?}"),
    }
    expect_started(start_instance(
        &application,
        definition.definition().clone(),
        132,
    ));
    expect_stale_start(start_instance(
        &application,
        definition.definition().clone(),
        133,
    ));
}

#[test]
fn public_authoring_publishes_replays_and_revises_one_branch_lineage() {
    let application = publish_workflow_on_first_program();

    let first = expect_published(
        "initial publication",
        publish_definition(
            &application,
            reviewed_geometry_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            1,
        ),
    );
    assert!(!first.replayed());
    let first_definition = first.definition().clone();

    let replay = expect_published(
        "exact replay",
        publish_definition(
            &application,
            reviewed_geometry_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            1,
        ),
    );
    assert!(replay.replayed());
    assert_eq!(replay.definition(), &first_definition);
    assert_eq!(
        replay.receipt().product_branch(),
        first.receipt().product_branch()
    );

    let second = expect_published(
        "lawful revision",
        publish_definition(
            &application,
            reviewed_geometry_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(first_definition.clone()),
            2,
        ),
    );
    assert!(!second.replayed());
    assert_eq!(second.definition().branch(), first_definition.branch());
    assert_ne!(
        second.definition().entity_id(),
        first_definition.entity_id()
    );
    assert_ne!(
        second.definition().content_identity(),
        first_definition.content_identity()
    );

    expect_stale_predecessor(publish_definition(
        &application,
        reviewed_geometry_definition("superseded"),
        WorkflowDefinitionExpectedPredecessor::Published(first_definition),
        3,
    ));

    expect_intent_drift(publish_definition(
        &application,
        reviewed_geometry_definition("different-intent"),
        WorkflowDefinitionExpectedPredecessor::Published(second.definition().clone()),
        1,
    ));
}

#[test]
fn public_instance_start_binds_revisions_replays_and_enforces_lineage_capacity() {
    let application = publish_workflow_on_first_program();
    let first = expect_published(
        "initial publication",
        publish_definition(
            &application,
            reviewed_geometry_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            11,
        ),
    );
    let first_definition = first.definition().clone();
    let started = expect_started(start_instance(&application, first_definition.clone(), 21));
    assert!(!started.replayed());
    assert_eq!(started.instance().current_node_path(), "propose");
    assert_eq!(
        started.instance().definition_content_identity(),
        first_definition.content_identity()
    );

    let replay = expect_started(start_instance(&application, first_definition.clone(), 21));
    assert!(replay.replayed());
    assert_eq!(
        replay.instance().entity_id(),
        started.instance().entity_id()
    );

    let second_start = expect_started(start_instance(&application, first_definition.clone(), 22));
    assert_ne!(
        second_start.instance().entity_id(),
        started.instance().entity_id(),
        "a new logical start must create a distinct instance"
    );

    let second = expect_published(
        "replacement publication",
        publish_definition(
            &application,
            reviewed_geometry_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(first_definition.clone()),
            12,
        ),
    );
    let replay_after_replacement =
        expect_started(start_instance(&application, first_definition.clone(), 21));
    assert!(replay_after_replacement.replayed());
    assert_eq!(
        replay_after_replacement.instance().entity_id(),
        started.instance().entity_id(),
        "definition replacement must not hide an exact retained replay"
    );
    expect_stale_start(start_instance(&application, first_definition.clone(), 23));

    let third = expect_published(
        "same-content new revision",
        publish_definition(
            &application,
            reviewed_geometry_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Published(second.definition().clone()),
            13,
        ),
    );
    assert_eq!(
        third.definition().content_identity(),
        first_definition.content_identity()
    );
    assert_ne!(third.definition().entity_id(), first_definition.entity_id());
    expect_start_intent_drift(start_instance(&application, third.definition().clone(), 21));

    for key in 100..130 {
        expect_started(start_instance(
            &application,
            third.definition().clone(),
            key,
        ));
    }
    let replay_at_capacity = expect_started(start_instance(&application, first_definition, 21));
    assert!(replay_at_capacity.replayed());
    assert_eq!(
        replay_at_capacity.instance().entity_id(),
        started.instance().entity_id()
    );
    expect_stale_start(start_instance(
        &application,
        third.definition().clone(),
        1000,
    ));
}

fn expect_started(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) -> worth_query_host::facade::application_entry::PerformedWorkflowInstanceStart {
    match result.expect("workflow instance-start preparation must succeed") {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("workflow instance start failed: {other:?}"),
    }
}

fn expect_published(
    context: &str,
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) -> worth_query_host::facade::application_entry::PerformedWorkflowDefinitionPublication {
    match result.expect("the public workflow publication must prepare") {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        unexpected => {
            panic!("{context}: expected a published workflow definition, got {unexpected:?}")
        }
    }
}

fn expect_stale_predecessor(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) {
    match result {
        Ok(WorkflowDefinitionPublicationOutcome::Application(
            WorthQueryApplicationCommitOutcome::Stale(stale),
        )) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected stale predecessor refusal, got {unexpected:?}"),
    }
}

fn expect_stale_start(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) {
    match result.expect("the workflow start must reach commit comparison") {
        WorkflowInstanceStartOutcome::Application(WorthQueryApplicationCommitOutcome::Stale(
            stale,
        )) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected stale workflow instance start, got {unexpected:?}"),
    }
}

fn expect_start_intent_drift(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) {
    match result.expect("the changed workflow start intent must reach idempotency") {
        WorkflowInstanceStartOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
            denial,
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
        ),
        unexpected => {
            panic!("expected workflow start idempotency intent drift, got {unexpected:?}")
        }
    }
}

fn expect_intent_drift(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) {
    match result.expect("the changed workflow intent must reach idempotency") {
        WorkflowDefinitionPublicationOutcome::Application(
            WorthQueryApplicationCommitOutcome::Denied(denial),
        ) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
        ),
        unexpected => panic!("expected workflow idempotency intent drift, got {unexpected:?}"),
    }
}

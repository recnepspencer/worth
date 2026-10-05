//! A live instance decides over the definition revision it started under. A
//! new revision or a new lineage capacity changes what a new start reads, and
//! leaves a waiting instance nothing to reconsider.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowProgressOutcome,
};
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_host::facade::domain::WorthQueryApplicationWorkflowResourceCeiling;

use super::workflow_participant::expect_started;
use crate::document_retention_model::{
    host::{publish_on_first_program, DocumentWorkflowRuntime},
    programs::RetentionProgramP1,
    workflow::{
        advance_instance, expect_capacity_refused, expect_superseded_start, install_workflow_spec,
        prepare_second_program_adoption, publish_adoption, publish_definition,
        retain_workflow_with_resources, second_program_workflow_inventory, start_instance,
        terminal_definition,
    },
};

/// The installed ceiling with `live_instances` live instances per lineage.
fn ceiling(live_instances: u32) -> WorthQueryApplicationWorkflowResourceCeiling {
    WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        live_instances,
        128,
        256 * 1024,
    )
    .expect("the workflow installation limits are nonzero")
}

fn published(
    application: &DocumentWorkflowRuntime,
    terminal: &str,
    predecessor: WorkflowDefinitionExpectedPredecessor,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    match publish_definition(application, terminal_definition(terminal), predecessor, key)
        .expect("the definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the {terminal} definition did not publish: {other:?}"),
    }
}

#[test]
fn a_waiting_instance_keeps_its_pinned_revision_across_a_definition_and_a_capacity_change() {
    // The first program admits two live instances per lineage, the second three.
    let mut application = retain_workflow_with_resources(publish_on_first_program(), ceiling(2));
    let second_program = install_workflow_spec(
        application.installed_schema(),
        application
            .supported_program::<RetentionProgramP1>()
            .expect("P1 is rostered")
            .installed_program(),
        ceiling(3),
    );
    application
        .workflow_mut()
        .support_workflow_spec(second_program)
        .expect("the vocabulary belongs to a rostered program");
    let main = application.current_world();
    let first = published(
        &application,
        "completed",
        WorkflowDefinitionExpectedPredecessor::Absent,
        97_000,
    );
    let waiting = expect_started(start_instance(&application, first.clone(), 97_001));

    // The definition changes under the waiting instance.
    let second = published(
        &application,
        "settled",
        WorkflowDefinitionExpectedPredecessor::Published(first.clone()),
        97_002,
    );
    let current = expect_superseded_start(start_instance(&application, first.clone(), 97_003));
    assert_eq!(current, second, "a new start is sent to the new revision");
    let newer = expect_started(start_instance(&application, second.clone(), 97_004));
    assert_eq!(newer.definition_entity_id(), second.entity_id());
    assert_eq!(newer.start_node_path(), "settled");
    let pinned = |at: &str| {
        let inventory = second_program_workflow_inventory(&application, main);
        let live = inventory
            .instance(waiting.entity_id())
            .unwrap_or_else(|| panic!("{at}: the waiting instance is live"));
        assert_eq!(
            live.definition(),
            first.entity_id(),
            "{at}: the waiting instance keeps the revision it started under"
        );
    };
    pinned("after the definition change");
    expect_capacity_refused(start_instance(&application, second.clone(), 97_005));

    // The lineage capacity changes under it: the branch adopts the program
    // whose vocabulary admits a third live instance.
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));
    let third = expect_started(start_instance(&application, second.clone(), 97_006));
    assert_eq!(
        third.definition_entity_id(),
        second.entity_id(),
        "a new start reads the new capacity and the new revision"
    );
    expect_capacity_refused(start_instance(&application, second.clone(), 97_007));
    pinned("after the capacity change");

    // No waiting step reads lineage capacity: the pinned instance completes
    // at its own revision's node while its lineage is full.
    match advance_instance(&application, waiting, 97_008)
        .expect("pinned work prepares against its retained definition")
    {
        WorkflowProgressOutcome::Completed(transition) => {
            assert_eq!(transition.node_path(), "completed");
            assert!(transition.terminal());
        }
        other => panic!("the pinned instance did not complete: {other:?}"),
    }
    // Its completion released the pin it held, and only that one.
    expect_started(start_instance(&application, second.clone(), 97_009));
    expect_capacity_refused(start_instance(&application, second, 97_010));
}

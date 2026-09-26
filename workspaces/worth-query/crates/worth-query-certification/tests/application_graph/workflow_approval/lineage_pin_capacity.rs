//! Every live instance pins its definition revision, and one lineage holds at
//! most the installed number of pins. A full lineage refuses a new start with
//! a typed denial before any effect, while a retry of a recorded start still
//! replays. Publication takes no pin, a migration exchanges one pin for
//! another, and evicting compiled meaning never releases a pin.

use worth_query_host::facade::application_entry::WorthQueryWorkflowInstancePreparationDenial;
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

use super::super::bounded_dimension_model::{
    host::publish_on_first_program,
    workflow::{
        cancel_instance, continue_on_fork, expect_capacity_refused, migrate_instance,
        retain_workflow_with_resources,
    },
};
use super::fork_continuation::fork_of;
use super::instance_cancellation::cancelled;
use super::instance_migration::{perform_approved_effect, replace_definition, started};
use super::journey::approval_requirement;
use super::*;

const PINS: u32 = 2;

type StartResult =
    Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>;

/// A workflow host whose lineages each hold at most `PINS` live instances.
fn pinned(
    key: u64,
) -> (
    BoundedDimensionWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
) {
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        PINS,
        64,
        256 * 1024,
    )
    .expect("the workflow installation limits are nonzero");
    let application = retain_workflow_with_resources(publish_on_first_program(), resources);
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the reviewed definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the reviewed definition did not publish: {other:?}"),
    };
    (application, definition)
}

fn start(
    application: &BoundedDimensionWorkflowRuntime,
    definition: &PublishedWorkflowDefinitionRef,
    key: u64,
) -> StartResult {
    start_instance(application, definition.clone(), key)
}

/// The instance a start published, and whether it replayed.
fn admitted(outcome: StartResult) -> (PublishedWorkflowInstanceRef, bool) {
    let performed = started(outcome);
    (performed.instance().clone(), performed.replayed())
}

fn release_everything(application: &BoundedDimensionWorkflowRuntime) {
    let runtime = application.runtime();
    runtime.release_workflow_instance_progress_for_test();
    runtime.release_workflow_compilation_for_test();
}

#[test]
fn a_full_lineage_refuses_a_new_start_but_replays_a_recorded_one() {
    let (application, definition) = pinned(96_000);
    let (first, _) = admitted(start(&application, &definition, 96_001));
    let (second, _) = admitted(start(&application, &definition, 96_002));
    expect_capacity_refused(start(&application, &definition, 96_003));
    expect_capacity_refused(start(&application, &definition, 96_003));
    let (replayed, replay) = admitted(start(&application, &definition, 96_002));
    assert!(
        replay,
        "a retry of a recorded start replays on a full lineage"
    );
    assert_eq!(replayed, second);

    release_everything(&application);
    expect_capacity_refused(start(&application, &definition, 96_004));
    let (replayed, replay) = admitted(start(&application, &definition, 96_001));
    assert!(replay, "a cold retry of a recorded start replays");
    assert_eq!(replayed, first);

    let ended = cancelled(cancel_instance(&application, first, 96_010));
    assert!(!ended.replayed());
    let (_, replay) = admitted(start(&application, &definition, 96_003));
    assert!(
        !replay,
        "the refused key recorded nothing, and cancellation released a pin",
    );
    expect_capacity_refused(start(&application, &definition, 96_011));
}

#[test]
fn publication_takes_no_pin_and_eviction_releases_none() {
    let (application, definition) = pinned(96_100);
    let (first, _) = admitted(start(&application, &definition, 96_101));
    admitted(start(&application, &definition, 96_102));
    let successor = replace_definition(
        &application,
        definition.clone(),
        reviewed_geometry_definition("applied"),
        96_110,
    );
    expect_capacity_refused(start(&application, &successor, 96_111));

    release_everything(&application);
    let before = application.runtime().workflow_compilation_reuse_counters();
    proposal::published_proposal(&application, first.clone(), 96_120);
    let evicted = application.runtime().workflow_compilation_reuse_counters();
    assert_eq!(
        evicted.cold_misses(),
        before.cold_misses() + 1,
        "the pinned historical revision recompiles once after eviction",
    );
    assert_awaiting_assessment(&application, &first, 96_121);
    assert_eq!(
        application
            .runtime()
            .workflow_compilation_reuse_counters()
            .cold_misses(),
        evicted.cold_misses(),
        "then the pinned revision is warm again",
    );
    expect_capacity_refused(start(&application, &successor, 96_122));

    let (migrated, _) = admitted(migrate_instance(
        &application,
        first,
        successor.clone(),
        "propose",
        96_130,
    ));
    assert_eq!(migrated.definition_entity_id(), successor.entity_id());
    expect_capacity_refused(start(&application, &successor, 96_131));
}

#[test]
fn publication_with_a_free_pin_still_admits_one_start() {
    let (application, definition) = pinned(96_300);
    admitted(start(&application, &definition, 96_301));
    let successor = replace_definition(
        &application,
        definition,
        reviewed_geometry_definition("applied"),
        96_310,
    );
    let (_, replay) = admitted(start(&application, &successor, 96_311));
    assert!(!replay, "publishing the successor took no pin");
    expect_capacity_refused(start(&application, &successor, 96_312));
}

/// A migration takes the successor's pin and releases its source's. Ending
/// the successor frees room for exactly one start, which it would not if the
/// source still held a pin.
#[test]
fn a_migration_exchanges_exactly_one_pin() {
    let (application, definition) = pinned(96_400);
    let (first, _) = admitted(start(&application, &definition, 96_401));
    admitted(start(&application, &definition, 96_402));
    let successor = replace_definition(
        &application,
        definition,
        reviewed_geometry_definition("applied"),
        96_410,
    );
    let (migrated, _) = admitted(migrate_instance(
        &application,
        first,
        successor.clone(),
        "propose",
        96_420,
    ));
    expect_capacity_refused(start(&application, &successor, 96_421));
    assert!(!cancelled(cancel_instance(&application, migrated, 96_430)).replayed());
    let (_, replay) = admitted(start(&application, &successor, 96_431));
    assert!(!replay);
    expect_capacity_refused(start(&application, &successor, 96_432));
}

/// A fork copies the lineage's pins with its instances. A continuation there
/// exchanges its copy's pin for its successor's, and the source branch keeps
/// its own count.
#[test]
fn a_fork_continuation_exchanges_its_copys_pin() {
    let (application, definition) = pinned(96_500);
    let (first, _) = admitted(start(&application, &definition, 96_501));
    admitted(start(&application, &definition, 96_502));
    let fork = fork_of(&application, first.branch());
    let on_fork = definition.held_on(fork);
    expect_capacity_refused(start(&application, &on_fork, 96_503));
    let (successor, _) = admitted(continue_on_fork(
        &application,
        fork,
        first,
        definition.clone(),
        "propose",
        96_510,
    ));
    expect_capacity_refused(start(&application, &on_fork, 96_511));
    assert!(!cancelled(cancel_instance(&application, successor, 96_520)).replayed());
    let (_, replay) = admitted(start(&application, &on_fork, 96_521));
    assert!(!replay);
    expect_capacity_refused(start(&application, &on_fork, 96_522));
    expect_capacity_refused(start(&application, &definition, 96_530));
}

fn assert_awaiting_assessment(
    application: &BoundedDimensionWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) {
    match advance_instance(application, instance.clone(), key) {
        Ok(WorkflowProgressOutcome::AwaitingAssessment(_)) => {}
        other => panic!("the pinned instance must await its assessment: {other:?}"),
    }
}

/// Once a revision is compiled, a whole run of a further instance compiles
/// nothing. Its first observation is its only progress miss, which rebuilds
/// from no transitions, and every later step reads retained progress.
#[test]
fn warm_progression_has_no_cold_compilation_or_reconstruction() {
    let (application, definition) = pinned(96_200);
    admitted(start(&application, &definition, 96_201));
    let compiled = application.runtime().workflow_compilation_reuse_counters();
    let progress = application.runtime().workflow_instance_progress_counters();

    let (instance, _) = admitted(start(&application, &definition, 96_210));
    let (proposal, required, _) = approval_requirement(&application, instance.clone(), 96_210);
    match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        96_220,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected the approval to complete, got {other:?}"),
    }
    perform_approved_effect(&application, &instance, 96_230);
    match advance_instance(&application, instance, 96_240) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected the performed effect to complete the run, got {other:?}"),
    }

    let after_compiled = application.runtime().workflow_compilation_reuse_counters();
    let after_progress = application.runtime().workflow_instance_progress_counters();
    assert_eq!(after_compiled.cold_misses(), compiled.cold_misses());
    assert_eq!(after_compiled.cold_retains(), compiled.cold_retains());
    assert!(after_compiled.warm_hits() > compiled.warm_hits());
    assert_eq!(
        after_progress.cold_misses(),
        progress.cold_misses() + 1,
        "only the new instance's first observation misses",
    );
    assert_eq!(
        after_progress.cold_reconstruction_transition_visits(),
        progress.cold_reconstruction_transition_visits(),
        "a new instance rebuilds from no transitions",
    );
    assert!(after_progress.warm_hits() > progress.warm_hits());
}

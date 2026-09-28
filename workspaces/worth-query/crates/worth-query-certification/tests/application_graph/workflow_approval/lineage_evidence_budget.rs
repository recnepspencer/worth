//! One lineage retains a bounded amount of assessment evidence. An accepted
//! assessment charges the evidence it writes against the installed ceiling,
//! a successor inherits what its source retained, and an assessment whose
//! evidence would pass the ceiling is refused, warm or cold.

use worth_query_host::facade::application_entry::{
    WorthQueryWorkflowAssessmentAcceptanceDenial, WorthQueryWorkflowAssessmentDemandSettlement,
};
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

use super::super::document_retention_model::{
    host::publish_on_first_program,
    schema::DocumentRetentionQuery,
    workflow::{cancel_instance, continue_on_fork, retain_workflow_with_resources},
};
use super::fork_continuation::fork_of;
use super::instance_cancellation::cancelled;
use super::instance_migration::started;
use super::*;

const UNBOUNDED: u64 = 256 * 1024;

/// A workflow host whose lineages each retain at most `evidence_bytes`.
fn bounded(evidence_bytes: u64) -> DocumentWorkflowRuntime {
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        32,
        64,
        evidence_bytes,
    )
    .expect("the workflow installation limits are nonzero");
    retain_workflow_with_resources(publish_on_first_program(), resources)
}

/// A proposed instance awaiting its first assessment.
fn proposed(
    application: &DocumentWorkflowRuntime,
    key: u64,
) -> (PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef) {
    let definition = match publish_definition(
        application,
        reviewed_document_definition("completed"),
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
    let instance = match start_instance(application, definition.clone(), key + 1)
        .expect("the instance start prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    };
    proposal::published_proposal(application, instance.clone(), key + 2);
    (definition, instance)
}

type Settlement = WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>;

/// Accepts `settlement` under `key` and reports whether it replayed.
fn accept(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    settlement: &Settlement,
    key: u64,
) -> bool {
    match accept_assessment(application, instance.clone(), settlement, key) {
        Ok(WorkflowProgressOutcome::Completed(performed)) => performed.replayed(),
        other => panic!("expected an accepted assessment, got {other:?}"),
    }
}

/// Settles and accepts the next assessment, returning its settlement.
fn accepted(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) -> Settlement {
    let settlement = settle_assessment(application, instance.clone(), key);
    assert!(!accept(application, instance, &settlement, key + 1));
    settlement
}

fn evidence_refused(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) {
    let settlement = settle_assessment(application, instance.clone(), key);
    // The ceiling is enforced where the evidence settles, so its refusal is
    // the acceptance's own attempt denial rather than a prepared step's.
    let kind = match accept_assessment(application, instance.clone(), &settlement, key + 1) {
        Err(WorthQueryWorkflowAssessmentAcceptanceDenial::Attempt(denial)) => denial.kind(),
        other => panic!("expected a refused assessment, got {other:?}"),
    };
    assert_eq!(
        kind,
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceEvidenceCapacityUnavailable,
    );
}

fn assert_awaiting(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) {
    match advance_instance(application, instance.clone(), key) {
        Ok(WorkflowProgressOutcome::AwaitingAssessment(_)) => {}
        other => panic!("the instance must still await its assessment: {other:?}"),
    }
}

/// The evidence bytes one accepted assessment charges its lineage, read from
/// the warm progress counters of an unbounded host.
fn one_assessment_charge(key: u64) -> u64 {
    let application = bounded(UNBOUNDED);
    let (_, instance) = proposed(&application, key);
    let before = application.runtime().workflow_instance_progress_counters();
    accepted(&application, &instance, key + 3);
    let charge = application
        .runtime()
        .workflow_instance_progress_counters()
        .incremental_evidence_bytes()
        - before.incremental_evidence_bytes();
    assert!(charge > 0, "an accepted assessment retains its evidence");
    charge
}

#[test]
fn evidence_past_the_ceiling_is_refused_and_leaves_the_instance_waiting() {
    let application = bounded(1);
    let (_, instance) = proposed(&application, 95_000);
    evidence_refused(&application, &instance, 95_010);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    evidence_refused(&application, &instance, 95_020);
    assert_awaiting(&application, &instance, 95_030);
    let ended = cancelled(cancel_instance(&application, instance, 95_040));
    assert!(
        !ended.replayed(),
        "a refused assessment leaves the instance live"
    );
}

#[test]
fn a_lineage_retains_evidence_only_up_to_its_ceiling() {
    let charge = one_assessment_charge(95_100);
    // The first assessment fills the ceiling exactly and is admitted.
    let application = bounded(charge);
    let (_, instance) = proposed(&application, 95_100);
    let first = accepted(&application, &instance, 95_103);
    assert!(
        accept(&application, &instance, &first, 95_104),
        "a retry of the accepted assessment replays",
    );
    evidence_refused(&application, &instance, 95_110);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    assert!(
        accept(&application, &instance, &first, 95_104),
        "a cold retry of the accepted assessment replays",
    );
    evidence_refused(&application, &instance, 95_120);
    assert_awaiting(&application, &instance, 95_130);
}

#[test]
fn a_fork_continuation_inherits_the_evidence_its_source_retained() {
    let charge = one_assessment_charge(95_200);
    let application = bounded(charge + charge / 2);
    let (definition, instance) = proposed(&application, 95_200);
    accepted(&application, &instance, 95_203);
    let fork = fork_of(&application, instance.branch());
    let successor = started(continue_on_fork(
        &application,
        fork,
        instance,
        definition,
        "propose",
        95_210,
    ))
    .instance()
    .clone();
    proposal::published_proposal(&application, successor.clone(), 95_211);
    evidence_refused(&application, &successor, 95_220);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    evidence_refused(&application, &successor, 95_230);
}

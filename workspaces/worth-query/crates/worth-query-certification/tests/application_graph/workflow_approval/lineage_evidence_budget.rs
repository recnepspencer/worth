//! One lineage retains a bounded amount of assessment evidence. An accepted
//! assessment charges the evidence it writes against the installed ceiling,
//! a successor inherits what its source retained, and an assessment whose
//! evidence would pass the ceiling is refused, warm or cold.

use worth_query_host::facade::application_entry::{
    WorthQueryWorkflowAssessmentAcceptanceDenial, WorthQueryWorkflowAssessmentDemandSettlement,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowComponentLimits, ApplicationWorkflowControlOutcome,
    ApplicationWorkflowDefinitionBuilder, ValidatedWorkflowDefinition,
};
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

use super::super::document_retention_model::{
    host::publish_on_first_program,
    schema::DocumentRetentionQuery,
    workflow::{
        cancel_instance, continue_on_fork, definition_limits, retain_workflow_with_resources,
        ReviewedDocumentWorkflow, WorkflowDefinitionAuthoringOperation,
    },
};
use super::fork_continuation::fork_of;
use super::instance_cancellation::cancelled;
use super::instance_migration::started;
use super::*;

// A caller-installed finite policy, independent of any other host's identities
// or observed charge. Exact inclusive byte admission belongs to the owner court.
const EVIDENCE_BUDGET: u64 = 4 * 1024;

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
        assessment_chain(),
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

fn assessment_chain() -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "evidence-budget",
        definition_limits(),
    )
    .unwrap();
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    // The fixture supplies enough real assessments to reach a finite refusal;
    // how many are accepted is deliberately not an oracle.
    let assessments = (0..16)
        .map(|index| {
            builder
                .assessment::<DocumentRetentionQuery>(&format!("checks/{index}"))
                .unwrap()
        })
        .collect::<Vec<_>>();
    builder.start(&propose).control(
        &propose,
        ApplicationWorkflowControlOutcome::Completed,
        &assessments[0],
    );
    for assessment in &assessments {
        builder.proposal_for_assessment(&propose, assessment);
    }
    for pair in assessments.windows(2) {
        builder.control(
            &pair[0],
            ApplicationWorkflowControlOutcome::Completed,
            &pair[1],
        );
    }
    let completed = builder.terminal("completed").unwrap();
    builder.control(
        assessments.last().unwrap(),
        ApplicationWorkflowControlOutcome::Completed,
        &completed,
    );
    builder.finish().unwrap().validate().unwrap()
}

type Settlement = WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>;

/// Accepts `settlement` under `key` and reports whether it replayed.
fn accept(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    settlement: &Settlement,
    key: u64,
) -> bool {
    accept_or_evidence_refusal(application, instance, settlement, key)
        .expect("expected an accepted assessment")
}

fn accept_or_evidence_refusal(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    settlement: &Settlement,
    key: u64,
) -> Option<bool> {
    let runtime = application.runtime();
    let current_commit = || {
        runtime
            .with_host_advancement(|phase| {
                runtime
                    .on_branch(instance.branch())
                    .select(&phase)
                    .unwrap()
                    .product()
                    .selected_commit()
                    .clone()
            })
            .unwrap()
    };
    let before_commit = current_commit();
    let before_bytes = runtime
        .workflow_instance_progress_counters()
        .incremental_evidence_bytes();
    match accept_assessment(application, instance.clone(), settlement, key) {
        Ok(WorkflowProgressOutcome::Completed(performed)) => Some(performed.replayed()),
        Err(WorthQueryWorkflowAssessmentAcceptanceDenial::Attempt(denial)) => {
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceEvidenceCapacityUnavailable
            );
            assert_eq!(
                current_commit(),
                before_commit,
                "refusal publishes no partial evidence"
            );
            assert_eq!(
                runtime
                    .workflow_instance_progress_counters()
                    .incremental_evidence_bytes(),
                before_bytes,
                "refusal does not charge progress"
            );
            None
        }
        other => panic!("expected acceptance or evidence-capacity refusal, got {other:?}"),
    }
}

/// Spend this host's real remaining allowance without predicting field widths.
fn spend_until_refused(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    mut key: u64,
    inherited: u64,
) -> (u64, Option<(Settlement, u64)>) {
    let baseline = application
        .runtime()
        .workflow_instance_progress_counters()
        .incremental_evidence_bytes();
    let mut retained = inherited;
    let mut last = None;
    loop {
        let settlement = settle_assessment(application, instance.clone(), key);
        if let Some(replayed) =
            accept_or_evidence_refusal(application, instance, &settlement, key + 1)
        {
            assert!(!replayed, "a new assessment is accepted once");
            let cumulative = inherited
                + application
                    .runtime()
                    .workflow_instance_progress_counters()
                    .incremental_evidence_bytes()
                - baseline;
            assert!(
                cumulative > retained,
                "accepted evidence has a positive charge"
            );
            assert!(
                cumulative <= EVIDENCE_BUDGET,
                "the lineage stays within its installed policy"
            );
            retained = cumulative;
            last = Some((settlement, key + 1));
            key += 2;
        } else {
            return (retained, last);
        }
    }
}

fn evidence_refused(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) {
    let settlement = settle_assessment(application, instance.clone(), key);
    assert!(accept_or_evidence_refusal(application, instance, &settlement, key + 1).is_none());
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
    let application = bounded(EVIDENCE_BUDGET);
    let (_, instance) = proposed(&application, 95_100);
    let (retained, last) = spend_until_refused(&application, &instance, 95_103, 0);
    let (last, acceptance_key) = last.expect("the installed budget admits real evidence");
    let before_replay = application
        .runtime()
        .workflow_instance_progress_counters()
        .incremental_evidence_bytes();
    assert!(
        accept(&application, &instance, &last, acceptance_key),
        "a retry of the accepted assessment replays",
    );
    evidence_refused(&application, &instance, 95_170);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    assert!(
        accept(&application, &instance, &last, acceptance_key),
        "a cold retry of the accepted assessment replays",
    );
    assert_eq!(
        application
            .runtime()
            .workflow_instance_progress_counters()
            .incremental_evidence_bytes(),
        before_replay,
        "warm and cold replay retain no new evidence"
    );
    assert!(retained <= EVIDENCE_BUDGET);
    evidence_refused(&application, &instance, 95_180);
    assert_awaiting(&application, &instance, 95_190);
}

#[test]
fn a_fork_continuation_inherits_the_evidence_its_source_retained() {
    let application = bounded(EVIDENCE_BUDGET);
    let (definition, instance) = proposed(&application, 95_200);
    let (inherited, last) = spend_until_refused(&application, &instance, 95_203, 0);
    assert!(
        last.is_some(),
        "the source retains real evidence before its fork"
    );
    let fork = fork_of(&application, instance.branch());
    let successor = started(continue_on_fork(
        &application,
        fork,
        instance,
        definition,
        "propose",
        95_300,
    ))
    .instance()
    .clone();
    proposal::published_proposal(&application, successor.clone(), 95_301);
    spend_until_refused(&application, &successor, 95_310, inherited);
    evidence_refused(&application, &successor, 95_350);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    evidence_refused(&application, &successor, 95_360);
    assert_awaiting(&application, &successor, 95_370);
}

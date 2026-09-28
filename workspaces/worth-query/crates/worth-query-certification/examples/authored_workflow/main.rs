//! Run with `cargo run -p worth-query-certification --example authored_workflow`.
//!
//! Authors a reviewed-document workflow from one reusable review component:
//! a proposed change to a document's retention period is checked twice and
//! approved before it applies. The example publishes the definition,
//! discovers it, rejects the first proposal so the loop asks for a revised
//! one, approves the revision and completes. A second revision of the
//! definition reuses the same component; discovery then names it, and a start
//! from the superseded revision is refused naming the current one.

#[allow(dead_code, unused_imports)]
#[path = "../../tests/application_graph/document_retention_model.rs"]
mod document_retention_model;
mod review;

use document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    readback::read_retention,
    retention_entry::{ReviewedSetRetentionBinding, ReviewedSetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
    workflow::{
        accept_assessment, advance_instance, approve_instance, definition_limits,
        expect_superseded_start, propose_authoring_instance_with_retention, publish_definition,
        settle_assessment, start_instance, ReviewedDocumentWorkflow, WorkflowApprovalCapability,
        WorkflowDefinitionAuthoringOperation,
    },
};
use review::{review, Review};
use worth_query_host::facade::{
    application_discovery::WorthQueryWorkflowDefinitionDiscovery,
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
        RequiredWorkflowApproval, WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
        WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome,
        WorkflowProgressOutcome, WorkflowProposalOutcome, WorthQueryApplicationMutationOutcome,
        WorthQueryApplicationRequestExt,
    },
    declaration::application_program::{
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
        ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowRetry,
        ValidatedWorkflowDefinition,
    },
};

fn main() {
    std::thread::Builder::new()
        .name("authored-workflow".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .expect("the bounded example thread must start")
        .join()
        .expect("the authored workflow must not unwind");
}

/// A rejected approval asks for a revised proposal, at most twice; the
/// completion terminal is all that differs between revisions.
fn reviewed_document(
    review: &Review,
    completion: &str,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut workflow = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "authored-reviewed-document",
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let propose = workflow
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("the proposal is valid");
    let approval = workflow
        .approval::<WorkflowApprovalCapability>("approval")
        .expect("the approval is valid");
    let apply = workflow
        .operation_binding::<ReviewedSetRetentionBinding>("apply")
        .expect("the guarded effect is valid");
    let completed = workflow
        .terminal(completion)
        .expect("the terminal is valid");
    let rejected = workflow
        .terminal("rejected")
        .expect("the terminal is valid");
    let checks = workflow
        .expand_component("checks", &review.component)
        .expect("the review expands");
    let structural = checks.input(&review.structural).expect("the input binds");
    let independent = checks.input(&review.independent).expect("the input binds");
    let evidence = checks.output(&review.evidence).expect("the output binds");
    let revise = ApplicationWorkflowRetry::new(
        ApplicationWorkflowControlOutcome::Rejected,
        "revise the proposal",
        2,
    )
    .expect("the revision bound is valid");
    workflow
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &structural,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &approval,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &rejected,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &apply,
        )
        .retry(&approval, revise, &propose)
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::RetryExhausted,
            &rejected,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .proposal_for_assessment(&propose, &structural)
        .proposal_for_assessment(&propose, &independent)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    workflow
        .finish()
        .expect("the reviewed workflow closes")
        .validate()
        .expect("the reviewed workflow is valid")
}

fn run() {
    let application = publish_workflow_on_first_program();
    let review = review();
    let first = reviewed_document(&review, "applied");
    let identity = first.identity().clone();
    assert_eq!(first.component_expansions().len(), 1);
    assert_eq!(
        discover(&application, &identity),
        WorthQueryWorkflowDefinitionDiscovery::Unpublished
    );
    let first = publish(
        &application,
        first,
        WorkflowDefinitionExpectedPredecessor::Absent,
        1,
    );
    assert_eq!(
        discover(&application, &identity),
        WorthQueryWorkflowDefinitionDiscovery::Current(first.clone())
    );

    let instance = match start_instance(&application, first.clone(), 2)
        .expect("the discovered definition starts")
    {
        WorkflowInstanceStartOutcome::Started(started) => started.instance().clone(),
        other => panic!("the instance must start, got {other:?}"),
    };
    let first_review = review_proposal(&application, &instance, 8, 100);
    assert_eq!(
        first_review.assessed,
        ["checks/structural", "checks/independent"],
        "a first proposal is reviewed in full"
    );
    match approve_instance(
        &application,
        instance.clone(),
        &first_review.required,
        &first_review.proposal,
        WorkflowApprovalDecision::Reject,
        200,
    )
    .expect("the rejection prepares")
    {
        WorkflowProgressOutcome::Completed(rejection) => {
            assert_eq!(rejection.node_path(), "approval")
        }
        other => panic!("the rejection must perform, got {other:?}"),
    }

    let revision = review_proposal(&application, &instance, 9, 300);
    assert_ne!(
        revision.proposal, first_review.proposal,
        "the loop reviews a revised proposal"
    );
    assert_eq!(
        revision.assessed,
        ["checks/structural", "checks/independent"],
        "a changed proposal is reviewed afresh"
    );
    match approve_instance(
        &application,
        instance.clone(),
        &revision.required,
        &revision.proposal,
        WorkflowApprovalDecision::Approve,
        400,
    )
    .expect("the approval prepares")
    {
        WorkflowProgressOutcome::Completed(approval) => {
            assert_eq!(approval.node_path(), "approval")
        }
        other => panic!("the approval must perform, got {other:?}"),
    }
    apply_approved_retention(&application, &instance, 9, 401);
    assert_eq!(advance(&application, &instance, 403), "applied");

    let second = publish(
        &application,
        reviewed_document(&review, "settled"),
        WorkflowDefinitionExpectedPredecessor::Published(first.clone()),
        500,
    );
    assert_eq!(
        discover(&application, &identity),
        WorthQueryWorkflowDefinitionDiscovery::Current(second.clone())
    );
    let current = expect_superseded_start(start_instance(&application, first, 501));
    assert_eq!(
        current, second,
        "a refused start names what discovery names"
    );
    println!("authored, reused, rejected, revised, approved, completed and discovered");
}

fn publish(
    application: &DocumentWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedDocumentWorkflow>,
    predecessor: WorkflowDefinitionExpectedPredecessor,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    match publish_definition(application, definition, predecessor, key)
        .expect("the definition prepares for publication")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => {
            published.definition().clone()
        }
        other => panic!("the definition must publish, got {other:?}"),
    }
}

fn discover(
    application: &DocumentWorkflowRuntime,
    identity: &ApplicationWorkflowDefinitionIdentity,
) -> WorthQueryWorkflowDefinitionDiscovery {
    application
        .runtime()
        .on_branch(application.runtime().current_world())
        .select()
        .expect("the main branch selects its exact occurrence")
        .discover_workflow_definition::<ReviewedDocumentWorkflow>(identity)
        .expect("the workflow lineage reads within its bound")
}

/// A proposed retention reviewed up to its approval.
struct Reviewed {
    proposal: PublishedWorkflowProposalRef,
    required: RequiredWorkflowApproval,
    /// The review nodes that collected fresh evidence for this proposal.
    assessed: Vec<String>,
}

/// Proposes a retention of `days`, then follows the instance through the
/// review until it awaits approval, collecting evidence wherever a review
/// demands it. Uses keys from `key` upward.
fn review_proposal(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    days: u64,
    key: u64,
) -> Reviewed {
    let proposal =
        match propose_authoring_instance_with_retention(application, instance.clone(), key, days)
            .expect("the proposal prepares")
        {
            WorkflowProposalOutcome::Published(published) => published.proposal().clone(),
            other => panic!("the proposal must publish, got {other:?}"),
        };
    let mut assessed = Vec::new();
    for key in (key + 1..).step_by(2).take(8) {
        match advance_instance(application, instance.clone(), key).expect("the review prepares") {
            WorkflowProgressOutcome::AwaitingAssessment(awaiting) => {
                let settlement = settle_assessment(application, instance.clone(), key);
                match accept_assessment(application, instance.clone(), &settlement, key + 1) {
                    Ok(WorkflowProgressOutcome::Completed(_)) => {}
                    other => panic!("the review must accept its evidence, got {other:?}"),
                }
                assessed.push(awaiting.node_path().to_owned());
            }
            WorkflowProgressOutcome::Completed(_) => {}
            WorkflowProgressOutcome::AwaitingApproval(required) => {
                return Reviewed {
                    proposal,
                    required,
                    assessed,
                };
            }
            other => panic!("the review must progress toward approval, got {other:?}"),
        }
    }
    panic!("the review must reach its approval within its bounded steps")
}

/// Applies the approved retention through the requirement the instance
/// awaits; the change's commit also settles the `apply` step.
fn apply_approved_retention(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    days: u64,
    key: u64,
) {
    let required = match advance_instance(application, instance.clone(), key)
        .expect("the approved effect requirement prepares")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("the approved effect must be awaited, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: days,
            },
        })
        .without_source()
        .idempotency(&(key + 1))
        .for_workflow_operation(application, &required)
        .expect("the effect matches the approved requirement")
        .execute_in_program(application.program_runtime())
        .expect("the approved effect executes");
    assert!(matches!(
        effect,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(read_retention(runtime, instance.branch()), days);
}

/// Advances one step and names the node it completed.
fn advance(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) -> String {
    match advance_instance(application, instance.clone(), key).expect("the step prepares") {
        WorkflowProgressOutcome::Completed(performed) => performed.node_path().to_owned(),
        other => panic!("the step must perform, got {other:?}"),
    }
}

//! Native uncertainty resolves the original key; aborted work cannot reissue its occurrence.
use super::{host, model, program::*};
use model::{operator_identity::*, retention_entry::*, schema::*};
use worth_query_host::facade::application_entry::*;
type Owner = WorthQueryWorkflowOperation<
    DocumentRetentionSchema,
    SetRetentionInput,
    WorkflowOwnerProgram,
    OccurrenceAction,
>;
fn original(host: &host::Host, key: u64) -> (PublishedWorkflowInstanceRef, Owner) {
    let instance = host.begin(
        super::super::workflow_execution::definition::<OccurrenceSetRetentionBinding>().unwrap(),
        8,
        key,
    );
    let required = match host.advance(instance.clone(), key + 3) {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        _ => panic!("original occurrence"),
    };
    let scope = request_scope();
    let actor = authenticate_operator(host.workflow.runtime().installed_schema(), &scope);
    let owner = host
        .workflow
        .runtime()
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .mutate(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.into(),
            retention_days: 8,
        })
        .without_source()
        .idempotency_owned(key + 4)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .for_workflow_operation(&host.workflow, &required)
        .unwrap_or_else(|_| panic!("original binding"))
        .retain_operation::<WorkflowOwnerProgram, OccurrenceAction>(host.workflow.program_runtime())
        .unwrap_or_else(|_| panic!("installed action"));
    (instance, owner)
}
#[test]
fn workflow_owning_indeterminate_resolves_original_key_without_handler_retry() {
    let host = host::install();
    let (instance, owner) = original(&host, 0x9176c500);
    let runtime = host.workflow.runtime();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope).on_branch(instance.branch());
    let guard = runtime
        .hold_post_commit_snapshot_failure_for_test()
        .unwrap();
    let owner = match owner.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Progress(owner) => owner,
        _ => panic!("real indeterminate outcome"),
    };
    let owner = match owner.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Refused { operation, .. } => operation,
        _ => panic!("held native boundary refuses exact keyed resolution"),
    };
    drop(guard);
    drop(request);
    drop(actor);
    drop(scope);
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope).on_branch(instance.branch());
    let completed = match owner.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Completed(done) => done,
        _ => panic!("fresh native original-key resolution"),
    };
    assert!(matches!(completed.evidence(),WorthQuerySourceCompletion::Recovered{initial_posture:Some(worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::Indeterminate(_)),..}));
    assert_eq!(
        model::readback::read_retention(runtime, instance.branch()),
        8
    );
    assert!(matches!(
        host.advance(instance, 0x9176c505),
        WorkflowProgressOutcome::Completed(_)
    ));
    completed
        .close(&request)
        .unwrap_or_else(|_| panic!("native resolved cleanup"));
}
#[test]
fn workflow_owning_proved_abort_never_reissues_spent_occurrence() {
    let host = host::install();
    let (instance, owner) = original(&host, 0x9176c600);
    let runtime = host.workflow.runtime();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope).on_branch(instance.branch());
    let before = model::readback::read_retention(runtime, instance.branch());
    let guard = runtime
        .reject_next_commit_before_transaction_for_test()
        .unwrap();
    let owner = match owner.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Progress(owner) => owner,
        _ => panic!("native aborted outcome retained"),
    };
    drop(guard);
    let owner = match owner.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Progress(owner) => owner,
        _ => panic!("aborted plus actual Unseen proves source had no effect"),
    };
    let terminal = match owner.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Terminal(terminal) => terminal,
        _ => panic!("spent occurrence cannot authorize a second handler attempt"),
    };
    assert!(matches!(
        terminal.denial(),
        Some(WorthQuerySourceOnlyDenial::Mutation(
            WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent
        ))
    ));
    assert!(
        matches!(
            terminal.original_abort(),
            Some(
                worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::Aborted
            )
        ),
        "authentic original abort survives keyed reset and spent-slot refusal"
    );
    assert_eq!(
        model::readback::read_retention(runtime, instance.branch()),
        before
    );
    assert!(matches!(
        host.advance(instance, 0x9176c605),
        WorkflowProgressOutcome::AwaitingOperation(_)
    ));
    terminal
        .close(&request)
        .unwrap_or_else(|_| panic!("native unexecuted owner cleanup"));
}

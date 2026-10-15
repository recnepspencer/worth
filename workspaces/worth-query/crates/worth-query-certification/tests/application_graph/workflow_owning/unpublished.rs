//! Real append failure retains the original attempt, never another source execution.
use super::{host, model, program::*};
use model::{operator_identity::*, retention_entry::*, schema::*};
use worth_query_host::facade::application_entry::*;
#[test]
fn workflow_owning_unpublished_recovers_original_binding() {
    let host = host::install();
    let runtime = host.workflow.runtime();
    let instance = host.begin(
        super::super::workflow_execution::definition::<OccurrenceSetRetentionBinding>().unwrap(),
        8,
        0x9176c200,
    );
    let required = match host.advance(instance.clone(), 0x9176c203) {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        _ => panic!("native occurrence"),
    };
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope).on_branch(instance.branch());
    let owner = request
        .mutate(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.into(),
            retention_days: 8,
        })
        .without_source()
        .idempotency_owned(0x9176c204_u64)
        .for_workflow_operation(&host.workflow, &required)
        .unwrap_or_else(|_| panic!("exact original binding"))
        .retain_operation::<WorkflowOwnerProgram, OccurrenceAction>(host.workflow.program_runtime())
        .unwrap_or_else(|_| panic!("installed action"));
    let before = model::readback::read_retention(runtime, instance.branch());
    let policy = crate::test_request_policy::serial_request();
    runtime.fail_next_durable_append_for_test(&worth_execution::ExecutionRequest::serial(&policy));
    let mut owner = match owner.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Progress(owner) => owner,
        _ => panic!("actual unpublished attempt retained"),
    };
    assert_eq!(
        model::readback::read_retention(runtime, instance.branch()),
        before
    );
    let cancellation=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(),
        );
    owner = match owner.advance(
        &runtime
            .request(&actor, &stopped)
            .on_branch(instance.branch()),
    ) {
        WorthQueryWorkflowOperationAdvance::Refused { operation, .. } => operation,
        _ => panic!("fresh refusal must preserve unpublished custody"),
    };
    // Bounded public advances delegate the native recovery and keyed owners, not the handler.
    let mut completed = None;
    for _ in 0..4 {
        let fresh_scope = request_scope();
        let fresh_actor = authenticate_operator(runtime.installed_schema(), &fresh_scope);
        match owner.advance(
            &runtime
                .request(&fresh_actor, &fresh_scope)
                .on_branch(instance.branch()),
        ) {
            WorthQueryWorkflowOperationAdvance::Progress(next) => owner = next,
            WorthQueryWorkflowOperationAdvance::Completed(done) => {
                completed = Some(done);
                break;
            }
            _ => panic!("native retained transition recovery"),
        }
    }
    let completed =
        completed.expect("native recovery completes within its existing bounded phases");
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered { .. }
    ));
    assert_eq!(
        model::readback::read_retention(runtime, instance.branch()),
        8
    );
    assert!(matches!(
        host.advance(instance.clone(), 0x9176c205),
        WorkflowProgressOutcome::Completed(_)
    ));
    let catalog = host
        .workflow
        .program_runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(catalog.rows().len(), 1);
    let inspector = host
        .workflow
        .program_runtime()
        .readmit_product_publication_recovery(catalog.rows()[0].handle())
        .unwrap();
    let held = inspector.inspect().unwrap();
    let (completed, denial) = completed
        .close(&request)
        .err()
        .expect("actual held inspector retains cleanup");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::Recovery(_)
    ));
    drop(held);
    completed
        .close(&request)
        .unwrap_or_else(|_| panic!("native recovery record close"));
}

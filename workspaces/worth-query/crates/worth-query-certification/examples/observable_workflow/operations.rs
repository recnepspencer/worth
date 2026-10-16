//! Original effect owners survive dispatch and native settlement; cleanup stays independent.
use crate::{
    document_retention_model::{
        operator_identity::*, programs::OccurrenceAction, retention_entry::DOCUMENT_IDENTITY,
        workflow::*, MixedApplication,
    },
    program::MixedProgram,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::WorthQueryInboundVerifierHandle,
};

pub fn capture(application: &MixedApplication, required: RequiredWorkflowOperation) {
    let scope = request_scope();
    let actor = authenticate_operator(application.runtime().installed_schema(), &scope);
    let request = application
        .runtime()
        .request(&actor, &scope)
        .on_branch(required.branch());
    let mut owner = request
        .mutate(crate::journey::input())
        .without_source()
        .idempotency_owned(40_u64)
        .unwrap_or_else(|(_, _, cause)| {
            panic!("capture retains original input/key on refusal: {cause:?}")
        })
        .for_workflow_operation(application, &required)
        .unwrap_or_else(|_| panic!("exact occurrence-only capture"))
        .retain_operation::<MixedProgram, OccurrenceAction>(application.program_runtime())
        .unwrap_or_else(|_| panic!("installed capture action"));
    loop {
        match owner.advance(&request) {
            WorthQueryWorkflowOperationAdvance::Progress(next) => owner = next,
            WorthQueryWorkflowOperationAdvance::Completed(done) => {
                done.close(&request)
                    .unwrap_or_else(|_| panic!("capture's native cleanup must close"));
                break;
            }
            WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("capture refused with original custody: {denial:?}");
            }
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
                let _native_custody = terminal;
                panic!("capture reached native terminality");
            }
        }
    }
}

pub fn archive(
    application: &MixedApplication,
    instance: &PublishedWorkflowInstanceRef,
    required: RequiredWorkflowOperation,
    archive: &crate::archive::ArchiveTransport,
    verifier: &WorthQueryInboundVerifierHandle,
) {
    let scope = request_scope();
    let actor = authenticate_operator(application.runtime().installed_schema(), &scope);
    let request = application
        .runtime()
        .request(&actor, &scope)
        .on_branch(required.branch());
    // compiled-example: begin
    let operation = request
        .mutate(crate::effect::ArchiveIntent {
            input: crate::journey::input(),
        })
        .without_source()
        .idempotency_owned(80_u64)
        .unwrap_or_else(|(_, _, cause)| panic!("original archive capture: {cause:?}"))
        .for_workflow_operation(application, &required)
        .unwrap_or_else(|_| panic!("exact approved occurrence"))
        .retain_operation::<MixedProgram, crate::effect::ArchiveAction>(
            application.program_runtime(),
        )
        .unwrap_or_else(|_| panic!("installed archive action"));
    let operation = match operation.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Progress(operation) => operation,
        _ => panic!("acknowledgement cannot settle the original dispatch"),
    };
    let (mut operation, cause) = operation
        .close(&request)
        .err()
        .expect("DispatchPending retains original native owner");
    assert!(matches!(
        cause,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));
    let waiting = crate::journey::advance(application, instance, &[81, 82]);
    assert!(
        matches!(waiting.stop(), WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingOperation(need)) if need.node_path() == "archive")
    );
    let envelope = archive
        .envelope()
        .expect("actual child file write and authenticated pipe completion");
    application
        .runtime()
        .inbound_occurrences()
        .receive(verifier, &envelope, &scope)
        .execute()
        .expect("native provenance admission of the real completion");
    let completed = loop {
        match operation.advance(
            &application
                .runtime()
                .request(&actor, &scope)
                .on_branch(instance.branch()),
        ) {
            WorthQueryWorkflowOperationAdvance::Progress(next) => operation = next,
            WorthQueryWorkflowOperationAdvance::Completed(done) => break done,
            WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("native settlement refused with original custody: {denial:?}");
            }
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
                let _original_custody = terminal;
                panic!("native settlement stopped terminally");
            }
        }
    };
    let accepted = request
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .without_source()
        .idempotency(&90_u64)
        .prepare_workflow_advance(application, instance.clone())
        .unwrap()
        .accept_owned_operation(completed)
        .unwrap_or_else(|(_owner, cause)| panic!("exact original external settlement: {cause:?}"));
    assert!(
        matches!(&accepted.progress, WorkflowProgressOutcome::Completed(performed) if performed.node_path() == "archive")
    );
    accepted
        .operation
        .close(&request)
        .unwrap_or_else(|_| panic!("original dispatch and native cleanup must close"));
    // compiled-example: end
}

/// Native producer output is admitted as evidence only at its exact waiting node.
#[inline(never)]
pub fn assess(
    application: &MixedApplication,
    instance: &PublishedWorkflowInstanceRef,
    path: &str,
    demand_key: u64,
    acceptance_key: u64,
) {
    use crate::document_retention_model::assessment_output::RetentionAssessmentDemand;
    use std::num::NonZeroUsize;
    let runtime = application.runtime();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope).on_branch(instance.branch());
    let mut owner = request
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .without_source()
        .idempotency(&demand_key)
        .prepare_workflow_advance(application, instance.clone())
        .expect("fresh admission at the waiting assessment")
        .into_assessment_demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .expect("installed retention query and producer contract")
        .controls(WorthQueryOutputDemandControls::new(
            NonZeroUsize::new(512).unwrap(),
            NonZeroUsize::new(1024).unwrap(),
        ))
        .retain_operation(application.program_runtime())
        .expect("retain the actual assessment producer owner");
    let settlement = match owner
        .advance(&request)
        .expect("advance the native producer")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settlement) => settlement,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the bounded installed producer must settle before evidence acceptance")
        }
    };
    owner
        .close(&request)
        .unwrap_or_else(|_| panic!("assessment cleanup retains failed custody"));
    let accepted = request
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .without_source()
        .idempotency(&acceptance_key)
        .prepare_workflow_advance(application, instance.clone())
        .expect("fresh exact assessment acceptance")
        .accept_assessment(&settlement)
        .expect("native settlement matches this instance and node");
    assert!(
        matches!(accepted, WorkflowProgressOutcome::Completed(ref done)
        if done.node_path() == path),
        "exact native assessment transition: {accepted:?}"
    );
}

/// Evaluate the authored condition from a native published read, never a supplied boolean.
#[inline(never)]
pub fn condition(
    application: &MixedApplication,
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowCondition,
) {
    use crate::document_retention_model::retention_entry::{
        DocumentRetentionConditionQueryBinding, DocumentRetentionConditionRead,
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let result = runtime
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .query(DocumentRetentionConditionRead {
            identity: DOCUMENT_IDENTITY.into(),
        })
        .execute()
        .expect("publish the real condition operand on this exact occurrence");
    assert_eq!(
        result.rows(),
        &[true],
        "native retention must satisfy the condition"
    );
    let accepted = runtime
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .without_source()
        .idempotency(&49_u64)
        .prepare_workflow_advance(application, instance.clone())
        .expect("fresh admission at the original condition requirement")
        .condition(required)
        .operand::<DocumentRetentionConditionQueryBinding, _>("retained", result)
        .accept()
        .expect("native condition evaluates current published proof for this requirement");
    assert!(
        matches!(accepted, WorkflowProgressOutcome::Completed(ref done)
        if done.node_path() == "positive-retention"),
        "the real condition transition must be performed: {accepted:?}"
    );
}

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
    let mut operation = request
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
    let receive = |envelope: &[u8]| {
        application
            .runtime()
            .inbound_occurrences()
            .receive(verifier, envelope, &scope)
            .execute()
            .expect("native provenance admission of the real completion");
    };
    let mut ingress_received = false;
    let completed = loop {
        match operation.advance(
            &application
                .runtime()
                .request(&actor, &scope)
                .on_branch(instance.branch()),
        ) {
            WorthQueryWorkflowOperationAdvance::Progress(next) => {
                operation = next;
                // Progress can precede dispatch. Actual child output, not this
                // variant, determines when ingress must precede further recovery.
                if !ingress_received
                    && let Some(envelope) = archive.envelope().expect(
                        "inspect actual child completion without retrying a failed dispatch",
                    )
                {
                    operation = exercise_pending_custody(operation, application, archive);
                    receive(&envelope);
                    ingress_received = true;
                }
            }
            WorthQueryWorkflowOperationAdvance::Completed(done) => {
                if !ingress_received {
                    let envelope = archive
                        .envelope()
                        .expect("inspect actual synchronous child completion")
                        .expect("native completion must retain genuine authenticated child output");
                    receive(&envelope);
                }
                break done;
            }
            WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("archive advance refused with original custody: {denial:?}");
            }
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
                panic!(
                    "archive terminal: source={:?}, recovery={:?}, abort={:?}",
                    terminal.denial(),
                    terminal.recovery_denial(),
                    terminal.original_abort()
                );
            }
        }
    };
    assert!(
        ingress_received,
        "the original owner must exercise pending custody before ingress/completion"
    );
    assert_eq!(archive.dispatch_calls(), 1, "one original child dispatch");
    assert_eq!(
        crate::effect::handler_invocations(),
        [1, 1],
        "one original handler execution"
    );
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

/// Exercise only the real authenticated-child / not-yet-ingressed native wait.
#[inline(never)]
fn exercise_pending_custody(
    operation: WorthQueryWorkflowOperation<
        crate::document_retention_model::schema::DocumentRetentionSchema,
        crate::effect::ArchiveIntent,
        MixedProgram,
        crate::effect::ArchiveAction,
    >,
    application: &MixedApplication,
    archive: &crate::archive::ArchiveTransport,
) -> WorthQueryWorkflowOperation<
    crate::document_retention_model::schema::DocumentRetentionSchema,
    crate::effect::ArchiveIntent,
    MixedProgram,
    crate::effect::ArchiveAction,
> {
    use worth_query_host::facade::{
        admission::authenticated_principal::{
            WorthQueryCancellationSource, WorthQueryRequestScope,
        },
        application_contribution::{
            WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
        },
    };
    let runtime = application.runtime();
    let branch = operation.required().branch();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope).on_branch(branch);
    let before = request.retain_current_read().unwrap();
    let envelope = archive
        .envelope()
        .unwrap()
        .expect("actual authenticated child output");
    let calls = (
        archive.dispatch_calls(),
        crate::effect::handler_invocations(),
    );
    assert_eq!(
        calls,
        (1, [1, 1]),
        "one actual dispatch and handler execution"
    );
    let (operation, cause) = operation
        .close(&request)
        .err()
        .expect("unresolved native inbound custody must not close");
    assert!(matches!(
        cause,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let operation = match operation.advance(&runtime.request(&actor, &stopped).on_branch(branch)) {
        WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
            assert!(
                matches!(
                    denial,
                    WorthQueryWorkflowOperationDenial::Source(
                        WorthQuerySourceOnlyDenial::Execution(
                            WorthQueryAdvancementDenial::Interrupted(
                                WorthQueryManagedComputationInterruption::Cancelled
                            )
                        )
                    )
                ),
                "actual typed cancellation: {denial:?}"
            );
            operation
        }
        _ => panic!("cancelled fresh advance must return original pending custody"),
    };
    let fresh_scope = request_scope();
    let fresh_actor = authenticate_operator(runtime.installed_schema(), &fresh_scope);
    let fresh = runtime
        .request(&fresh_actor, &fresh_scope)
        .on_branch(branch);
    let operation = match operation.advance(&fresh) {
        WorthQueryWorkflowOperationAdvance::Progress(operation) => operation,
        WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
            let _original_owner = operation;
            panic!("legal original wait refused: {denial:?}");
        }
        _ => panic!("without native ingress the original effect must still wait"),
    };
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    assert_eq!(archive.envelope().unwrap().as_ref(), Some(&envelope));
    assert_eq!(
        (
            archive.dispatch_calls(),
            crate::effect::handler_invocations()
        ),
        calls,
        "close/cancel/fresh wait must not redispatch or rerun the handler"
    );
    operation
}

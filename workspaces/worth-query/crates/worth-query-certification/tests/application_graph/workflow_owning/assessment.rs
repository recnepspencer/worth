//! Source save, observation, evidence acceptance and approval are distinct native steps.
use super::{host, model, program::*};
use model::{operator_identity::*, retention_entry::*, schema::*, workflow::*};
use worth_query_host::facade::application_entry::*;
#[test]
fn workflow_owning_source_completion_does_not_accept_assessment() {
    let host = host::install();
    let runtime = host.workflow.runtime();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope);
    let source = request
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.into(),
                retention_days: 9,
            },
        })
        .without_source()
        .idempotency_owned(0x9176c300_u64)
        .retain_operation::<WorkflowOwnerProgram, SavedSourceAction>(
            host.workflow.program_runtime(),
        )
        .unwrap_or_else(|_| panic!("declared source save"));
    let WorthQuerySourceOnlyAdvance::Completed(saved) = source.advance(&request) else {
        panic!("real source save")
    };
    assert_eq!(
        model::readback::read_retention(runtime, runtime.current_world()),
        9
    );
    saved
        .close(&request)
        .unwrap_or_else(|_| panic!("source cleanup"));
    let instance = host.begin(reviewed_document_definition("applied"), 9, 0x9176c301);
    let (observer, settlement) = host.assessment(instance.clone(), 0x9176c304);
    assert!(
        matches!(
            host.advance(instance.clone(), 0x9176c305),
            WorkflowProgressOutcome::AwaitingAssessment(_)
        ),
        "observation cannot record evidence or approve"
    );
    let cancel=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancel.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancel.token(),
        );
    let (observer, _) = observer
        .close(
            &runtime
                .request(&actor, &stopped)
                .on_branch(instance.branch()),
        )
        .err()
        .expect("outer admission refusal retains actual assessment observation");
    observer
        .close(&request)
        .unwrap_or_else(|_| panic!("observer close, not approval"));
    assert!(
        matches!(
            host.advance(instance.clone(), 0x9176c306),
            WorkflowProgressOutcome::AwaitingAssessment(_)
        ),
        "closing observation does not settle workflow"
    );
    assert!(matches!(
        host.accept_assessment(instance, &settlement, 0x9176c307),
        WorkflowProgressOutcome::Completed(_)
    ));
}

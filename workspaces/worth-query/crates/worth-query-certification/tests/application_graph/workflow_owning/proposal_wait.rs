//! A proposal slot is a native wait, never a made-up transition or unsupported node.
use super::{host, model};
use model::{operator_identity::*, retention_entry::DOCUMENT_IDENTITY, workflow::*};
use worth_query_host::facade::application_entry::*;

#[test]
fn ordinary_run_reports_the_actual_proposal_need_without_publishing_a_transition() {
    let host = host::install();
    let instance =
        host.begin_without_proposal(reviewed_document_definition("applied"), 9, 0x9176d100);
    let runtime = host.workflow.runtime();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope).on_branch(instance.branch());
    let before = request.retain_current_read().unwrap();
    let retention = model::readback::read_retention(runtime, runtime.current_world());
    let keys = [0x9176d103_u64, 0x9176d104_u64];
    for key in keys {
        let supplied_keys = [key];
        let progress = request
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    document_identity: DOCUMENT_IDENTITY.into(),
                },
            })
            .run_workflow(&host.workflow, instance.clone())
            .idempotency_keys(&supplied_keys)
            .execute();
        let WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingProposal(
            need,
        )) = progress.stop()
        else {
            panic!("a real proposal slot must be a typed wait, got {progress:?}");
        };
        let operation = WorkflowDefinitionAuthoringOperation::reference();
        assert_eq!(need.node_path(), "propose");
        assert_eq!(need.operation(), operation.name());
        assert_eq!(need.input_type(), operation.input_identity().as_str());
        assert!(progress.transitions().is_empty());
        assert_eq!(progress.attempted_steps(), 1);
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            before.selected_commit()
        );
        assert_eq!(
            model::readback::read_retention(runtime, runtime.current_world()),
            retention
        );
    }
    let proposal = request
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: DOCUMENT_IDENTITY.into(),
                retention_days: 9,
            },
        })
        .without_source()
        .idempotency(&0x9176d105_u64)
        .prepare_workflow_proposal(&host.workflow, instance.clone())
        .unwrap()
        .execute();
    assert!(
        matches!(proposal, WorkflowProposalOutcome::Published(_)),
        "fresh real proposal admission consumes its native slot"
    );
    assert!(
        matches!(
            host.advance(instance, 0x9176d106),
            WorkflowProgressOutcome::AwaitingAssessment(_)
        ),
        "the same native workflow now reaches its real successor"
    );
}

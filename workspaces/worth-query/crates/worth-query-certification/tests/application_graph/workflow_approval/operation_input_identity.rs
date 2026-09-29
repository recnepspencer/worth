//! A workflow operation encodes its input once, at bind, and reports the
//! request's two identity derivations in its admission, first and replayed.

use super::*;
use crate::canonical_identity::assert_request_identity_work;
use crate::document_retention_model::schema::SetRetentionInput;

#[test]
fn a_workflow_operation_encodes_its_input_once_whether_it_commits_or_replays() {
    let (application, _, instance, proposal, approval, _) = approval_journey("applied", 2_300);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            2_310,
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 2_311).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an operation requirement, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let perform = || {
        runtime
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .mutate(ReviewedSetRetentionIntent {
                input: SetRetentionInput {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                    retention_days: 8,
                },
            })
            .without_source()
            .idempotency(&2_312_u64)
            .for_workflow_operation(&application, &required)
            .expect("the exact proposed operation binds")
            .execute_in_program(application.program_runtime())
            .expect("the approved operation executes")
    };

    SetRetentionInput::reset_encoding_count();
    let first = perform();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = first else {
        panic!("the approved operation commits: {first:?}");
    };
    assert_eq!(
        SetRetentionInput::encoding_count(),
        1,
        "binding to the requirement, admission and the commit share one input encoding"
    );
    // The key 2_312 (3 encoded bytes, 113 hashed with the 64-byte reviewed
    // command namespace, 2 blocks) and the input (60 encoded bytes, 163 hashed
    // with the 57-byte input type, 3 blocks).
    assert_request_identity_work(receipt.canonical_work().admission(), 63, 276, 5);

    SetRetentionInput::reset_encoding_count();
    let retry = perform();
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the unchanged retry replays: {retry:?}");
    };
    assert_eq!(
        SetRetentionInput::encoding_count(),
        1,
        "a retry encodes its input once too"
    );
    assert_request_identity_work(recovered.canonical_work().admission(), 63, 276, 5);
}

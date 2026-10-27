//! Every expected history coordinate comes from an ordinary, actually committed command.
use super::super::{
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::{SetRetentionIntent, DOCUMENT_IDENTITY},
    workflow::{WorkflowGrantStatusInput, WorkflowGrantStatusIntent},
};
use super::{access::App, *};
use worth_query_host::facade::{
    application_entry::*, primary_graph::WorthQueryApplicationCommitReceipt,
    product::WorthQueryProductBranch,
};

pub(super) fn retention(
    application: &App,
    branch: WorthQueryProductBranch,
    value: u64,
    key: u64,
    reaffirm: bool,
) -> WorthQueryApplicationCommitReceipt {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: value,
    };
    let outcome = runtime
        .with_application_advancement(&scope, |phase| {
            if reaffirm {
                runtime
                    .request(&principal, &scope)
                    .on_branch(branch)
                    .advanced()
                    .mutate(reaffirm::Intent(input))
                    .without_source()
                    .idempotency(&key)
                    .execute_in_program_in_advancement(&phase, application)
            } else {
                runtime
                    .request(&principal, &scope)
                    .on_branch(branch)
                    .advanced()
                    .mutate(SetRetentionIntent { input })
                    .without_source()
                    .idempotency(&key)
                    .execute_in_program_in_advancement(&phase, application)
            }
        })
        .unwrap()
        .unwrap();
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed {
            receipt, result, ..
        } => {
            assert_eq!(result.retention_days, value);
            receipt
        }
        other => panic!("authored retention must commit: {other:?}"),
    }
}
pub(super) fn grant(
    application: &App,
    identity: &str,
    status: &str,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .with_application_advancement(&scope, |phase| {
            runtime
                .request(&principal, &scope)
                .advanced()
                .mutate(WorkflowGrantStatusIntent {
                    input: WorkflowGrantStatusInput {
                        grant_identity: identity.to_owned(),
                        status: status.to_owned(),
                    },
                })
                .without_source()
                .idempotency(&key)
                .execute_in_program_in_advancement(&phase, application)
        })
        .unwrap()
        .unwrap();
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("actual grant status must change: {other:?}"),
    }
}

//! Fresh ordinary request recovery reads require the current program.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRecoveryRequestDenial, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

use super::document_retention_model::{
    host::{publish_on_first_program, publish_on_second_program},
    operator_identity::{authenticate_operator, request_scope},
    programs::RemovedOperationRetentionProgram,
    readback::observe_head,
    retention_entry::{ReviewedSetRetentionIntent, SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

fn intent(retention_days: u64) -> SetRetentionIntent {
    SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days,
        },
    }
}

#[test]
fn recovery_reads_require_the_current_program_and_refuse_workflow_bound_actions() {
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let target = *host
        .supported_program::<RemovedOperationRetentionProgram>()
        .unwrap()
        .owned_revision();
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let comparison = programs.compare(&target).unwrap();
    assert!(matches!(
        programs
            .adopt(&comparison)
            .unwrap()
            .prepare(64)
            .unwrap()
            .publish(runtime, &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    let before = observe_head(runtime, branch).selected_commit().clone();
    let key = 0xaced_1802;
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(branch)
            .advanced()
            .mutate(intent(9))
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(&host),
        Err(WorthQueryApplicationRecoveryRequestDenial::Request(
            WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired
        ))
    ));
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(branch)
            .advanced()
            .mutate(ReviewedSetRetentionIntent {
                input: intent(9).input
            })
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(&host),
        Err(WorthQueryApplicationRecoveryRequestDenial::WorkflowUnsupported)
    ));
    let foreign = publish_on_second_program();
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(branch)
            .advanced()
            .mutate(intent(9))
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(&foreign),
        Err(WorthQueryApplicationRecoveryRequestDenial::Request(
            WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch
        ))
    ));
    assert_eq!(observe_head(runtime, branch).selected_commit(), &before);
}

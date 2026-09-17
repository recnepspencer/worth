use bank_domain::{estate::EstateAction, proposals::BankIdempotencyKey};
use bank_estate_certification::{
    exercise_estate_lifecycle, EstateLifecycleInputs, EstateLifecyclePrincipals,
    EstateLifecycleProgressionOutcome,
};
use bank_server::{BankEstateProgressionDenial, BankIdentityRuntime};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

#[allow(dead_code)]
fn public_consumer_compiles_one_move_only_lifecycle(
    runtime: &BankIdentityRuntime,
    principals: EstateLifecyclePrincipals<'_>,
    request: &WorthQueryRequestScope,
    actions: [EstateAction; 4],
    idempotency: [BankIdempotencyKey; 4],
) -> Result<(), BankEstateProgressionDenial> {
    let [request_action, approval_action, close_action, review_action] = actions;
    let outcome = exercise_estate_lifecycle(
        runtime,
        principals,
        request,
        EstateLifecycleInputs {
            request_action,
            approval_action,
            close_action,
            review_action,
            idempotency,
        },
    )?;
    match outcome {
        EstateLifecycleProgressionOutcome::Reviewed(receipt)
        | EstateLifecycleProgressionOutcome::AlreadyReviewed(receipt) => {
            let _ = receipt;
        }
        EstateLifecycleProgressionOutcome::RequestStopped(outcome) => {
            let _ = outcome;
        }
        EstateLifecycleProgressionOutcome::ApprovalStopped(outcome) => {
            let _ = outcome;
        }
        EstateLifecycleProgressionOutcome::CloseStopped(outcome) => {
            let _ = outcome;
        }
        EstateLifecycleProgressionOutcome::ReviewStopped(outcome) => {
            let _ = outcome;
        }
    }
    Ok(())
}

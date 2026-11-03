//! Closing recorded retry bookkeeping does not dispose the actual effect owner.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryWorkflowEffectRecoveryDenial as EffectDenial,
    WorthQueryWorkflowOperationAdvance as Advance, WorthQueryWorkflowOperationDenial as Denial,
};

#[test]
fn recorded_workflow_retry_closes_without_disposing_original_dispatch_custody() {
    let ready = ReadyPaymentWorld::new(
        "recorded-workflow-close",
        FaultScript::CommitThenLoseResponse,
    );
    let original = ready
        .perform()
        .into_performed()
        .expect("original payment performed");
    let runtime = &ready.fixture.world.runtime;
    let workflow = runtime.approved_business_payment(&ready.principal, &ready.scope);
    let perform_key = key("approved-payment:operation:perform");
    let request = runtime
        .request(&ready.principal, &ready.scope)
        .on_branch(ready.operation.branch());
    let duplicate = workflow
        .retain_apply(
            &ready.operation,
            ready.authority.clone(),
            perform_key.clone(),
        )
        .unwrap();
    let duplicate = match duplicate.advance(&request) {
        Advance::Refused { operation, denial } => {
            let Denial::Recovery(EffectDenial::Owner(worth_query_host::facade::application_entry::WorthQueryWorkflowOperationOwnerPosture::RecoveredCommitted(data))) = denial else {
                panic!("recorded retry must disclose data without dispatch permission: {denial:?}");
            };
            assert_eq!(
                data.commit_reference(),
                original.correction_source().relational()
            );
            assert_eq!(data.emitted_effect_count(), original.emitted_effect_count());
            let head = data.new_product_head().memory().unwrap();
            assert_eq!(
                head.owner(),
                original.correction_source().world().owner_identity().get()
            );
            assert_eq!(
                head.selected_commit(),
                original.correction_source().world().ordinal()
            );
            operation
        }
        Advance::Progress(_) | Advance::Completed(_) | Advance::Terminal(_) => {
            panic!("recorded source must not settle or mint a dispatch owner")
        }
    };
    duplicate
        .close(&request)
        .unwrap_or_else(|(_owner, cause)| panic!("only duplicate cleanup closes: {cause:?}"));
    assert_eq!(ready.rail.attempts().len(), 1);
    assert_eq!(ready.rail.completed_effect_count(), 1);
    let recovery = workflow
        .advanced()
        .prepare_native_effect_recovery(
            &ready.operation,
            &original,
            ready.authority.clone(),
            &perform_key,
        )
        .expect("duplicate close leaves the genuine original recoverable");
    drop(recovery);
    assert_eq!(
        ready.rail.attempts().len(),
        1,
        "claim observation does not send again"
    );
    assert_eq!(ready.rail.completed_effect_count(), 1);
}

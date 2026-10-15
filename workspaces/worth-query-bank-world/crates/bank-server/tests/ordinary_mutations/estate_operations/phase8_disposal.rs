//! Real notification recovery disposal preserves unique custody through fresh refusals.
use super::phase8_cross_gate::world::{cross_gate_world, PATIENT};
use crate::support::request_scope;
use bank_external_rail::test_control::FaultScript;
use bank_server::{
    BankAuthorizationDenialKind, BankEstateProgressionDenial, BankRecoveryClaimStatus,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::WorthQueryApplicationRequestMutationDenial,
};

#[test]
fn disposal_returns_original_handle_on_refusal_then_native_receipt() {
    let world = cross_gate_world("owning-disposal-fresh-authority");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_notification(131);
    let runtime = &world.fixture.world.runtime;
    let handle = world.open_recovery(&receipt);
    let revision = world.estate_account_revision();
    let notice = world.notice_status();
    let attempts = world.transport.attempts();
    let effects = world.transport.completed_effect_count();
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        effects, 1,
        "real rail effect already landed despite lost response"
    );
    assert_eq!(
        runtime.commit_recovery_claim_status(&receipt),
        Ok(BankRecoveryClaimStatus::Live)
    );

    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let canceled = WorthQueryRequestScope::new(None, cancellation.token());
    let specialist = world.fixture.authenticate_specialist();
    let (cause, handle) = runtime
        .dispose_commit_recovery(handle, &specialist, world.specialist_action(), &canceled)
        .expect_err("cancelled opening must return the whole owner")
        .into_parts();
    assert!(matches!(
        cause,
        BankEstateProgressionDenial::ApplicationEntry(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        )
    ));
    let handle = handle.expect("opening refusal cannot consume native custody");
    assert_eq!(
        runtime.commit_recovery_claim_status(&receipt),
        Ok(BankRecoveryClaimStatus::Live)
    );

    let unauthorized = world.fixture.authenticate_deceased();
    let (cause, handle) = runtime
        .dispose_commit_recovery(
            handle,
            &unauthorized,
            world.specialist_action(),
            &request_scope(),
        )
        .expect_err("account ownership cannot replace the specialist capability")
        .into_parts();
    let BankEstateProgressionDenial::Authorization(cause) = cause else {
        panic!("expected actual fresh capability refusal, got {cause:?}");
    };
    assert_eq!(
        cause.kind(),
        BankAuthorizationDenialKind::CapabilityGrantMissing
    );
    let handle = handle.expect("admission refusal returns that original handle");
    assert_eq!(
        runtime.commit_recovery_claim_status(&receipt),
        Ok(BankRecoveryClaimStatus::Live)
    );

    let fresh_specialist = world.fixture.authenticate_specialist();
    let disposed = runtime
        .dispose_commit_recovery(
            handle,
            &fresh_specialist,
            world.specialist_action(),
            &request_scope(),
        )
        .expect("fresh lawful specialist disposes the returned original handle");
    let binding = disposed.native_disposal_receipt().binding();
    let original_product = receipt.publication().product_commit();
    assert_eq!(
        binding.commit_reference(),
        original_product.relational_commit()
    );
    assert_eq!(
        binding.branch(),
        &original_product.relational_commit().branch_id
    );
    assert_eq!(
        binding.committed_product_publication().composite_commit(),
        original_product.composite_commit()
    );
    assert_eq!(
        binding
            .committed_product_publication()
            .product_incarnation(),
        original_product.product_incarnation()
    );
    assert_eq!(
        binding.installed_aftermath_operation_slot(),
        "NotifyDeathEstateOperation"
    );
    assert_eq!(
        disposed.transition_receipt().installed_operation(),
        binding.installed_aftermath_operation_slot()
    );
    assert_eq!(
        runtime.commit_recovery_claim_status(&receipt),
        Ok(BankRecoveryClaimStatus::Disposed)
    );
    assert_eq!(world.transport.attempts(), attempts);
    assert_eq!(world.transport.completed_effect_count(), effects);
    assert_eq!(world.estate_account_revision(), revision);
    assert_eq!(
        world.notice_status(),
        notice,
        "disposal is not business rollback or another source effect"
    );
    let (_description, actual_native_receipt) = disposed.into_parts();
    assert_eq!(
        actual_native_receipt.binding().commit_reference(),
        original_product.relational_commit()
    );
}

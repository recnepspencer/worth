//! Actual World originals identify owner claims without granting a new handle.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryClaimStatus as Status;
use crate::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

#[test]
fn admitted_original_status_tracks_only_its_actual_provider_and_world_claim() {
    let (world, live) = recoverable_application_world(239, "recorded-recovery-status");
    let original = super::super::fixture::recorded_receipt::read_original(&world, &live);
    let request = live_scope();
    let observe = || {
        world
            .application
            .with_host_advancement(|phase| {
                let principal =
                    authenticated_principal(&phase.test_host_request(), &world, &request);
                let account = resolved_account(
                    &phase.test_host_request(),
                    &world,
                    "recorded-recovery-status",
                    &request,
                );
                let operation = world
                    .application
                    .installed_schema()
                    .installed_operation(ExactStatusRetentionOperation::reference())
                    .unwrap();
                let admission = world
                    .selected_product(&phase.test_host_request())
                    .authorize_operation(
                        &phase,
                        &principal,
                        &account,
                        &operation,
                        TypedMutationPreconditions::new(),
                        &request,
                    )
                    .unwrap();
                world
                    .application
                    .resolve_admitted_recovery_claim_status(
                        &phase,
                        &admission,
                        idempotency(239, 240),
                    )
                    .unwrap()
            })
            .unwrap()
    };
    assert_eq!(observe(), Some(Status::Unclaimed));
    let handle = world.application.mint_recovery_handle(&live).unwrap();
    assert_eq!(observe(), Some(Status::Live));
    let registry = &world.application.recovery_handles;
    assert_eq!(
        registry.recorded_claim_state(
            live.provider_runtime_instance_id() + 1,
            original.new_product_head()
        ),
        (false, None)
    );
    let (sibling, sibling_live) = recoverable_application_world(239, "sibling-status");
    let sibling_original =
        super::super::fixture::recorded_receipt::read_original(&sibling, &sibling_live);
    assert_eq!(
        registry.recorded_claim_state(
            live.provider_runtime_instance_id(),
            sibling_original.new_product_head()
        ),
        (false, None)
    );
    drop(handle);
    assert_eq!(observe(), Some(Status::Disposed));
    assert_eq!(
        world.application.recovery_claim_status(&live).unwrap(),
        Status::Disposed
    );
}

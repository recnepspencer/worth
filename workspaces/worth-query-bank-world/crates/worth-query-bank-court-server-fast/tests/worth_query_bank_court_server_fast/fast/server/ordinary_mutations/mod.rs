pub(crate) use worth_query_bank_worlds::facade::server::ordinary_mutations::*;

pub(crate) mod estate_operations;

pub(crate) mod owning_accounts;

pub(crate) mod owning_deposit;

pub(crate) mod posting_lease;

pub(crate) mod preconditions;

pub(crate) mod program_families;

pub(crate) mod publication;

pub(crate) mod recorded_correction;

pub(crate) mod retained_publication;

pub(crate) mod transfer_posting_oracle;

#[test]
fn public_mutation_controls_preserve_interruptions_permissions_and_intent_drift() {
    use worth_query_host::facade::application_entry::{
        WorthQuerySourceCompletion as Completion, WorthQuerySourceOnlyAdvance as Advance,
        WorthQuerySourceOnlyDenial as Denial,
    };
    let fixture = ordinary_read_world("ordinary-mutation-controls", 0);
    let owner = fixture.authenticate(OWNER);
    let stranger = fixture.authenticate(STRANGER);
    let send = send_for(&fixture);
    for cancelled in [true, false] {
        let cancellation = WorthQueryCancellationSource::new();
        let capture_scope = request_scope();
        let scope = WorthQueryRequestScope::new(
            Some(if cancelled {
                Instant::now() + Duration::from_secs(60)
            } else {
                Instant::now() - Duration::from_secs(1)
            }),
            cancellation.token(),
        );
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::send_money(send.clone()))
            .as_principal(&owner)
            .controls(BankMutationControls::new(
                capture_scope,
                key(if cancelled { "cancelled" } else { "expired" }),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("construction: {cause:?}"));
        if cancelled {
            cancellation.cancel();
        }
        let request = fixture.world.runtime.request(&owner, &scope);
        let Advance::Refused {
            operation,
            denial: Denial::Execution(WorthQueryAdvancementDenial::Interrupted(interruption)),
        } = operation.advance(&request, Some(std::num::NonZeroUsize::new(20_000).unwrap()))
        else {
            panic!("exact request interruption")
        };
        assert_eq!(
            interruption,
            if cancelled {
                WorthQueryManagedComputationInterruption::Cancelled
            } else {
                WorthQueryManagedComputationInterruption::DeadlineExceeded
            }
        );
        let _runtime = operation.into_runtime_for_cleanup();
    }
    let scope = request_scope();
    let request = fixture.world.runtime.request(&stranger, &scope);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&stranger)
        .controls(BankMutationControls::new(scope.clone(), key("denied")))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("construction: {cause:?}"));
    let Advance::Refused {
        operation,
        denial: Denial::Mutation(denial),
    } = operation.advance(&request, Some(std::num::NonZeroUsize::new(20_000).unwrap()))
    else {
        panic!("native permission refusal")
    };
    assert!(matches!(denial.kind(), worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::Authorization(worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied))));
    let _runtime = operation.into_runtime_for_cleanup();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&owner, &scope);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&owner)
        .controls(BankMutationControls::new(scope.clone(), key("drift")))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("construction: {cause:?}"));
    let Advance::Completed(completed) =
        operation.advance(&request, Some(std::num::NonZeroUsize::new(20_000).unwrap()))
    else {
        panic!("fresh send")
    };
    let Completion::Fresh { receipt, .. } = completed.evidence() else {
        panic!("fresh evidence")
    };
    assert!(receipt.emitted_effect_count() > 0);
    assert!(completed.close(&request).is_ok());
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            amount: Money::from_minor(26).unwrap(),
            ..send
        }))
        .as_principal(&owner)
        .controls(BankMutationControls::new(scope.clone(), key("drift")))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("construction: {cause:?}"));
    let Advance::Terminal(terminal) =
        operation.advance(&request, Some(std::num::NonZeroUsize::new(20_000).unwrap()))
    else {
        panic!("intent drift terminal")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
    ));
    assert!(terminal.close(&request).is_ok());
}

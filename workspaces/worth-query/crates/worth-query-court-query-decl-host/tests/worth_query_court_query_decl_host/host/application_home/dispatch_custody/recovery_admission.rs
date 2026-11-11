//! Installed recovery requirements refuse before handler, commit, or rail contact.
use super::*;
use worth_foundational::facade::DurableStateFamily;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestMutationDenialKind,
    product::WorthQueryDurableStateAdmissionDenial,
};
use worth_query_testkit_host::facade::host::dispatch_home::recoverable::RecoverableNoticeIntent;

#[test]
fn a_recoverable_non_inbound_operation_refuses_before_effects() {
    let location = dispatch::location();
    let contacts = Arc::new(AtomicUsize::new(0));
    let rail = ScriptedRail::answering(&[Outcome::Completed]);
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&location, execution, &contacts, true, &rail);
    {
        let (_, scope) = pending_support::scope();
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let denial = runtime
            .request(&principal, &scope)
            .advanced()
            .mutate(RecoverableNoticeIntent(19))
            .without_source()
            .idempotency(&0x8f01_u64)
            .execute_in_program(
                runtime,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect_err("the installed recoverable aftermath requires durable claims");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationRequestMutationDenialKind::DurableStateAdmission(
                WorthQueryDurableStateAdmissionDenial::Unsupported {
                    family: DurableStateFamily::RecoveryHandleCustody,
                    supported_from_phase: 8
                },
            )
        );
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert!(rail.contacts.lock().unwrap().is_empty());
    // The same Home still serves the supported non-recoverable operation.
    {
        use worth_query_host::facade::primary_graph::{
            WorthQueryRecoveryClaimStatus as Status, WorthQueryRecoveryHandleDenialKind as Kind,
        };
        let (_, scope) = pending_support::scope();
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let outcome = runtime
            .request(&principal, &scope)
            .advanced()
            .mutate(NotifyAccountBalance { balance: 23 })
            .without_source()
            .idempotency(&0x8f02_u64)
            .execute_in_program(
                runtime,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("the non-recoverable operation is still supported");
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("the supported operation commits");
        };
        assert_eq!(
            runtime.runtime().recovery_claim_status(&receipt).unwrap(),
            Status::Unclaimed
        );
        let denial = runtime
            .runtime()
            .mint_recovery_handle(&receipt)
            .expect_err("Directory claim minting independently checks custody support");
        assert_eq!(
            denial.kind(),
            Kind::DurableStateAdmission(WorthQueryDurableStateAdmissionDenial::Unsupported {
                family: DurableStateFamily::RecoveryHandleCustody,
                supported_from_phase: 8,
            })
        );
        assert_eq!(
            runtime.runtime().recovery_claim_status(&receipt).unwrap(),
            Status::Unclaimed
        );
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(rail.contacts.lock().unwrap().len(), 1);
    Arc::into_inner(home)
        .expect("exclusive Home")
        .close(&execution)
        .unwrap_or_else(|refused| panic!("the supported Home still closes: {refused:?}"))
        .reclaim();
}

//! Managed recovery enters the same source boundary as its positive control.
use super::*;
#[test]
fn managed_recovery_refuses_before_revalidation_at_both_placements() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let world = installed_authorization_world(true);
        let request = live_scope();
        let principal = authenticated_principal(&world, &request);
        let account = resolved_account(&world, "open", &request);
        let program = admitted_program(&world, &principal, &account, &request, "recovered");
        world.application.fail_next_durable_append_for_test();
        let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) =
            world.application.compare_and_commit_application(
                program,
                idempotency(191, 191),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        else {
            panic!("real durability fault retains recovery")
        };
        let recovery = partial.into_recovery();
        recovery.continue_owner_settlement().unwrap();
        let fresh = admitted_operation(&world, &principal, &account, &request);
        for memory in [false, true] {
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                if memory { 0 } else { 64 * 1024 * 1024 },
                if memory { 8_000_000 } else { 0 },
            )));
            let before = reads();
            let Err(crate::domain_computation::primary_graph::WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied(cause)) = world.application.recover_admitted_unpublished_application(&recovery, &fresh, idempotency(191, 191)) else { panic!("recovery retains opening cause") };
            if !memory {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            } else if placement == Placement::Serial {
                assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit));
            } else {
                let Denial::Resource(Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                }) = cause
                else {
                    panic!("exact bytes")
                };
                assert_eq!(
                    level,
                    crate::domain_computation::primary_graph::WorthQueryMemoryLimitLevel::Policy
                );
                assert!(requested > 0);
                assert_eq!(admitted, 0);
            }
            assert_eq!(reads(), before);
        }
        bound(None);
        let before = reads();
        let outcome = world
            .application
            .recover_admitted_unpublished_application(&recovery, &fresh, idempotency(191, 191))
            .unwrap();
        assert!(matches!(outcome, crate::domain_computation::primary_graph::WorthQueryManagedApplicationRecoveryOutcome::Performed(_)));
        assert!(
            reads() > before,
            "admitted recovery contacts its exact source"
        );
    }
}

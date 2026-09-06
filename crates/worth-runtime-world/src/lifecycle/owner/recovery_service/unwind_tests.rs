use std::panic::{catch_unwind, AssertUnwindSafe};

use super::settlement_catalog_tests::{relational_attempt, setup, successor_basis};
use crate::lifecycle::RuntimeWorldRecoveryService;
use crate::publication::{
    CompositeAttemptProgress, RelationalAttemptProgress, SignalAttemptProgress,
};
use crate::recovery::ProductUnpublishedNextAction;

#[test]
fn settlement_unwind_restores_custody_before_catalog_reinstallation() {
    // Stop before route extraction and after real settlement. Neither case
    // may turn the catalog record into untouched or cleanup-only progress.
    for settle_before_unwind in [false, true] {
        let (fixture, owner, expected) = setup();
        let mut attempt = relational_attempt(&fixture, &owner, expected.clone());
        attempt.begin_owner_execution();
        let performed = fixture.perform_relational_owner_change();
        let basis = performed.next_basis().clone();
        let successor = successor_basis(&owner, &expected, basis.clone(), None);
        let retained = attempt
            .settle(CompositeAttemptProgress::new(
                RelationalAttemptProgress::performed(performed),
                SignalAttemptProgress::untouched(),
            ))
            .ready(successor)
            .expect_err("settlement remains owed");
        let commit = retained
            .component_results()
            .relational_publication_identity();
        let basis_identity = retained
            .component_results()
            .relational_publication_basis_identity()
            .cloned();
        let handle = retained.recovery_handle();
        drop(retained);
        let before = owner.state.branches.root_cell().unwrap().atomic_snapshot();
        let signal_before = owner
            .state
            .signal
            .basis_port()
            .owner_service_cost_snapshot()
            .unwrap();

        let unwind = catch_unwind(AssertUnwindSafe(|| {
            let mut update = owner
                .state
                .recovery
                .take_record_for_update(&handle)
                .unwrap();
            let mut custody = update
                .record_mut()
                .unwrap()
                .take_relational_recovery()
                .unwrap();
            if settle_before_unwind {
                let identity = custody
                    .take_identity_repair()
                    .expect("retained custody names actual owner repair");
                owner
                    .state
                    .relational
                    .settlement_port()
                    .repair_pending_publication_settlement(identity.commit_id())
                    .unwrap();
            }
            std::panic::panic_any("settlement caller unwound");
        }));
        assert_eq!(
            unwind.unwrap_err().downcast_ref::<&str>(),
            Some(&"settlement caller unwound")
        );
        let restored = owner
            .inspect_recovery(&handle)
            .expect("same record remains installed");
        assert!(restored.progress().relational_requires_settlement());
        assert_eq!(restored.recovery_handle(), handle);
        assert!(commit.is_some());
        assert_eq!(
            restored
                .component_results()
                .relational_publication_identity(),
            commit
        );
        assert_eq!(
            restored
                .component_results()
                .relational_publication_basis_identity(),
            basis_identity.as_ref()
        );
        assert!(restored
            .next_actions()
            .contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
        drop(restored);
        assert!(
            owner.cleanup_recovery_handle(&handle).is_none(),
            "unwind cannot permit premature cleanup"
        );
        let resumed = owner.inspect_recovery(&handle).unwrap();
        let continuation =
            RuntimeWorldRecoveryService::continue_effects(owner.as_ref(), resumed).unwrap();
        assert!(!continuation
            .actions()
            .contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
        assert_eq!(
            owner.state.branches.root_cell().unwrap().atomic_snapshot(),
            before
        );
        assert_eq!(
            owner
                .state
                .signal
                .basis_port()
                .owner_service_cost_snapshot()
                .unwrap()
                .canonical_movements(),
            signal_before.canonical_movements()
        );
        assert_eq!(
            owner
                .inspect_recovery(&handle)
                .unwrap()
                .progress()
                .relational_posture(),
            crate::publication::RelationalAttemptProgressPosture::Settled
        );
        assert!(owner.cleanup_recovery_handle(&handle).is_some());
    }
}

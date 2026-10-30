//! The creation-specific wrapper preserves both refusal owners and actual cleanup receipts.
use super::*;
use crate::basis::{
    WorthQueryProductBranchCreationRecoveryReleaseFailure as ReleaseFailure,
    WorthQueryProductBranchRecoveryDenial,
};

#[test]
fn creation_release_retains_live_recovery_and_defers_actual_owner_cleanup() {
    for deferred in [false, true] {
        let host_request = crate::test_request_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        let world = installed_authorization_world(true);
        let (recovery, occupied) = fork_failure::create(&world, execution);
        let inspection = recovery.inspect().unwrap();
        assert_eq!(
            inspection.cause(),
            WorthQueryProductBranchCreationRecoveryCause::SiblingOwnerDenied
        );
        assert_eq!(inspection.owner_effect_count(), 1);
        assert!(inspection.live_obligation_count() > 0);
        let page = world
            .application
            .branches()
            .recovery_page(None, NonZeroUsize::MIN)
            .unwrap();
        let [row] = page.rows() else {
            panic!("exactly one genuine creation recovery")
        };
        let second = world
            .application
            .branches()
            .readmit_recovery(row.handle())
            .unwrap();
        let ReleaseFailure::Recovery(failure) = recovery.release_obligations(0).unwrap_err() else {
            panic!("the independently readmitted live capability retains recovery")
        };
        assert_eq!(
            failure.denial(),
            WorthQueryProductBranchRecoveryDenial::CallerCapabilityLive
        );
        let recovery = failure.into_recovery();
        assert_eq!(recovery.inspect().unwrap().owner_effect_count(), 1);
        assert_eq!(
            world
                .application
                .branches()
                .recovery_page(None, NonZeroUsize::MIN)
                .unwrap()
                .rows()
                .len(),
            1
        );
        assert!(world.application.branches().pending_cleanup().is_empty());
        drop(second);
        drop(page);
        let graph = world.application.runtime.primary_graph().unwrap();
        let integration = graph.integration_handle();
        let active = deferred.then(|| {
            integration.with_runtime(execution, |runtime| {
                let identity = runtime
                    .branch_identity(&worth_relational::facade::history::BranchId(
                        "query-product-1-relational".to_owned(),
                    ))
                    .expect("the actual performed Relational fork");
                let (_, basis) = runtime.observe_branch(&identity).unwrap();
                runtime
                    .begin_branch_transaction(
                        &basis,
                        worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                    )
                    .expect("a real operation retains the fork during cleanup")
            })
        });
        if deferred {
            let ReleaseFailure::OwnerCleanup(failure) =
                recovery.release_obligations(0).unwrap_err()
            else {
                panic!("released World effects transfer their actual cleanup owner")
            };
            assert_eq!(
                failure.denial(),
                WorthQueryProductBranchOwnerCleanupDenial::RelationalOperationsStillActive
            );
            assert_eq!(world.application.branches().pending_cleanup().len(), 1);
            assert!(world
                .application
                .branches()
                .recovery_page(None, NonZeroUsize::MIN)
                .unwrap()
                .rows()
                .is_empty());
            let failure = failure.into_cleanup().retry().unwrap_err();
            assert_eq!(
                failure.denial(),
                WorthQueryProductBranchOwnerCleanupDenial::RelationalOperationsStillActive
            );
            drop(active);
            let receipt = failure.into_cleanup().retry().unwrap();
            assert_eq!(receipt.retired_component_count(), 1);
            assert!(receipt.is_complete());
        } else {
            let receipt = recovery.release_obligations(0).unwrap();
            assert_eq!(receipt.retired_component_count(), 1);
            assert!(receipt.is_complete());
            drop(active);
        }
        assert!(world.application.branches().pending_cleanup().is_empty());
        assert!(world
            .application
            .branches()
            .recovery_page(None, NonZeroUsize::MIN)
            .unwrap()
            .rows()
            .is_empty());
        let receipt = world
            .application
            .with_host_advancement(|phase| {
                world.application.on_branch(occupied).close(&phase).unwrap()
            })
            .unwrap();
        assert_eq!(receipt.retired_component_count(), 1);
        assert!(receipt.is_complete());
        assert!(world.application.branches().pending_cleanup().is_empty());
    }
}

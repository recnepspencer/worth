//! A valid physical cut still needs original and native source-slot funding.
use super::*;
use worth_store::physical_runtime::{
    PhysicalRecoveryRejoinResidentDenial, RecoveryWalAllocationDenial,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, PhysicalRecoverySourceDenial, PhysicalRecoveryStaticConfiguration,
    PhysicalRecoveryWalInventoryAllocationBoundary,
};

#[test]
fn selected_source_slot_requires_original_and_native_capacity_before_retaining_backing() {
    let (world, receipt, _) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
    assert!(receipt.remaining_payload_records() > 0);
    crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::checkpoint(world.serving(), [0xd6; 32]);
    let store = world.serving().store_identity();
    let retained = world.retained_root();
    let root = retained.path();
    drop(world);
    let before = snapshot_family(root);
    let source_bytes =
        std::mem::size_of::<worth_store_recovery_physics::PhysicalSourceSelection>() as u64;
    // Input DATA, its movement temporary and its new slot already require
    // three payload headers; native custody adds its own nonempty header.
    let insufficient = source_bytes.checked_mul(3).unwrap();
    let original = PhysicalRecoveryStaticConfiguration::current();
    let native = original
        .clone()
        .with_residency_policy(recovery_policy_with_scope(insufficient))
        .expect("actual native Recovery scope keeps the admitted format");
    for (index, (memory, configuration)) in [(insufficient, original), (24 << 20, native)]
        .into_iter()
        .enumerate()
    {
        let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
            worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_configuration(root, memory, configuration),
        );
        let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
            panic!("source slot must refuse before retaining unfunded backing: {outcome:?}");
        };
        assert_eq!(blocked.store_identity(), store);
        assert_eq!(blocked.recovery_effects(), 0);
        let evidence = blocked.evidence();
        assert_eq!(evidence.counters.current_root_integrity_admissions, 1);
        let [PhysicalRecoverySourceDenial::WalInventoryAllocation {
            boundary: PhysicalRecoveryWalInventoryAllocationBoundary::SourceSelection,
            cause: RecoveryWalAllocationDenial::Backing { requested, cause },
        }] = evidence.source_denials.as_slice()
        else {
            panic!(
                "exact selected-source allocation boundary: {:?}",
                evidence.source_denials
            );
        };
        assert!(*requested > insufficient);
        if index == 0 {
            let PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted } = cause
            else {
                panic!("original recovery ceiling remains the cause: {cause:?}");
            };
            assert!(*required >= *requested);
            assert_eq!(*admitted, insufficient);
        } else {
            let PhysicalRecoveryRejoinResidentDenial::OperationAllocation(cause) = cause else {
                panic!("native Recovery scope remains the cause: {cause:?}");
            };
            let pressure = cause.pressure().expect("native Recovery pressure");
            assert_eq!(pressure.basis().store_identity(), store);
            assert_eq!(
                pressure.scope(),
                worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery
            );
            assert_eq!(pressure.limit(), insufficient);
            assert!(!pressure.effect_may_have_started());
        }
        assert_eq!(snapshot_family(root), before);
    }
}

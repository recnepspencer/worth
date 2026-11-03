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
    // Keep this cut before its first checkpoint, so checkpoint decoding is
    // not an earlier, larger allocation boundary than the source slot.
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("selected-source-funding")
        .expect("real filesystem Store before its first checkpoint");
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
    let original = PhysicalRecoveryStaticConfiguration::current()
        .with_durability_declaration(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::serving_durability(
            std::num::NonZeroU64::new(16 << 20).unwrap(),
        ));
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
        let [PhysicalRecoverySourceDenial::RootProtocol {
            artifact:
                worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
            denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
        }, PhysicalRecoverySourceDenial::WalInventoryAllocation {
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

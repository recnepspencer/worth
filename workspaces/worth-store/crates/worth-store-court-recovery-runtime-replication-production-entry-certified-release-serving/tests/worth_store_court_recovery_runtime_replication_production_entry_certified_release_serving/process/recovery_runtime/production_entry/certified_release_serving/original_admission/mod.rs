//! Recovery carries one pool into Serving while retaining its original ceiling.
//! The larger or smaller target policy is declared before recovery admission.

#[allow(unused_imports)]
use crate::production_entry::certified_release_serving::original_admission::*;

#[test]
fn original_recovery_ceiling_survives_seal_checkpoint_and_fresh_reopen() {
    let (world, receipt, _) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(
            1024,
        );
    assert_eq!(receipt.remaining_payload_records(), 0);
    let store = world.serving().residency_observation().store_identity();
    checkpoint(world.serving(), [0xb1; 32]);
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("original-recovery-admission-serving".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let first = recover_seal(&root, store, worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current());
            let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(&root, first);
            let observation = serving.residency_observation();
            let target = observation.admitted_policy().scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
            assert!(target > ORIGINAL_RECOVERY_BYTES);
            assert_eq!(observation.store_identity(), store);
            let counters = observation.counters();
            assert_eq!(counters.peak_operation_bytes(), ORIGINAL_RECOVERY_BYTES);
            // Serving entry funds the selected custody's standing capture
            // reservation from the carried pool; nothing else stays active.
            let standing = serving
                .certification_checkpoint_capture_custody_bytes()
                .expect("Serving entry funds the selected capture reservation");
            assert_eq!(counters.active_operation_bytes(), standing);
            let grants = observation
                .allocations()
                .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationBytes);
            assert!(grants.admissions() > 0);
            assert!(grants.admitted_units() >= ORIGINAL_RECOVERY_BYTES);
            // The original ceiling must constrain the real Serving issuer,
            // not just a recovery sizing hint. The larger target policy would
            // admit this competing request without the carried native clamp.
            let issuer = serving.physical_allocations();
            let held = issuer
                .admit_recovery(std::num::NonZeroU64::MIN)
                .expect("one byte fits the genuine recovered pool");
            let before = serving.residency_observation().allocations();
            let denial =
                match issuer.admit_recovery(std::num::NonZeroU64::new(ORIGINAL_RECOVERY_BYTES).unwrap()) {
                    Err(denial) => denial,
                    Ok(_) => panic!("Serving cannot spend beyond the original aggregate ceiling"),
                };
            let pressure = denial
                .pressure()
                .expect("original native Recovery pressure");
            assert_eq!(pressure.scope(), worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
            assert_eq!(
                pressure.dimension(),
                worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery),
            );
            assert_eq!(pressure.requested(), ORIGINAL_RECOVERY_BYTES);
            assert_eq!(pressure.admitted(), standing + 1);
            assert_eq!(pressure.limit(), ORIGINAL_RECOVERY_BYTES);
            assert!(!pressure.effect_may_have_started());
            let after = serving.residency_observation().allocations();
            let dimension = worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
            assert_eq!(after.for_dimension(dimension).active_units(), standing + 1);
            assert_eq!(
                after.for_dimension(dimension).admitted_units(),
                before.for_dimension(dimension).admitted_units(),
            );
            drop(held);
            checkpoint(&serving, [0xb2; 32]);
            assert_eq!(
                serving
                    .residency_observation()
                    .counters()
                    .peak_operation_bytes(),
                ORIGINAL_RECOVERY_BYTES,
                "checkpoint work must not reopen the larger target operation grant"
            );
            assert!(!serving.close().residency().requires_inspection());

            // Admit the smaller target pool before fresh C8 recovery. Its
            // scope and the original recovery ceiling both continue to hold.
            let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            );
            let second = recover_seal(
                &root,
                store,
                worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current()
                    .with_residency_policy(lower_target_policy(format))
                    .expect("lower target policy admits the configured format"),
            );
            let serving = open_with_lower_target_policy(&root, second);
            let observation = serving.residency_observation();
            assert_eq!(observation.store_identity(), store);
            assert_eq!(
                observation.admitted_policy().scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery),
                LOWER_TARGET_BYTES
            );
            let peak = observation.counters().peak_operation_bytes();
            assert!(peak > 0 && peak < ORIGINAL_RECOVERY_BYTES);
            assert!(peak <= LOWER_TARGET_BYTES);
            checkpoint(&serving, [0xb3; 32]);
            assert!(
                serving
                    .residency_observation()
                    .counters()
                    .peak_operation_bytes()
                    <= LOWER_TARGET_BYTES
            );
            assert!(!serving.close().residency().requires_inspection());
        })
        .expect("recovery worker");
    worker.join().expect("recovery worker did not panic");
}

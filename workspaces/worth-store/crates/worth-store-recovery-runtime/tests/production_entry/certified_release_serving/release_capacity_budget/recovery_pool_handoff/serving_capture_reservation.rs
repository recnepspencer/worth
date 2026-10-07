//! Serving entry funds the selected ledger's standing capture reservation in
//! the carried pool; a pool that cannot fund it denies entry before effects.
use super::*;
use worth_store::physical_runtime::{
    PhysicalSignalConstructionFailure, RecordBootstrapFailure, ReleaseCertificateCapacityDenial,
};

#[test]
fn unfundable_capture_reservation_denies_serving_entry_before_any_effect() {
    std::thread::Builder::new()
        .name("serving-capture-reservation-denial".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            checkpoint(world.serving(), [0xe5; 32]);
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let dimension = Dimension::OperationScope(Scope::Recovery);

            // The genuine standing reservation of this media, from a healthy entry.
            let before_media = snapshot_family(root);
            let core = recover_core(root);
            let policy = core.residency_policy();
            let format = AdmittedPhysicalRecordFormat::admit(policy.record_format());
            let seal = core
                .into_checkpoint_custody()
                .expect("genuine release seal");
            let TransitionOutcome::Success(serving) =
                open_with_policy(root, seal, format, policy).into_raw()
            else {
                panic!("an unpressured pool must fund the reservation");
            };
            let standing = serving
                .certification_checkpoint_capture_custody_bytes()
                .expect("Serving entry funds the selected capture reservation");
            serving.close();
            assert_eq!(
                snapshot_family(root),
                before_media,
                "entry and close leave media unchanged"
            );

            // A peer leaves less than the reservation, even after the WAL
            // fingerprint is disposed, while earlier entry reads still fit.
            let core = recover_core(root);
            let original = core.recovery_allocation_admission().byte_limit();
            let observer = core.certification_residency_allocations();
            let retained_bytes = observer.snapshot().for_dimension(dimension).active_units();
            let fingerprint = backing_census::retained_fingerprint_bytes(&core);
            let peer_bytes = original - retained_bytes - standing + 1 + fingerprint;
            let peer = core
                .certification_begin_recovery_allocation(NonZeroU64::new(peer_bytes).unwrap())
                .unwrap();
            let seal = core.into_checkpoint_custody().expect("fresh genuine seal");
            let TransitionOutcome::Failed(inspection) =
                open_with_policy(root, seal, format, policy).into_raw()
            else {
                panic!("an unfundable reservation must not become Serving");
            };
            let RecordBootstrapFailure::SignalConstruction(
                PhysicalSignalConstructionFailure::ServingCaptureCustodyReservationRejected(
                    ReleaseCertificateCapacityDenial::Resident(
                        PhysicalRecoveryRejoinResidentDenial::OperationAllocation(allocation),
                    ),
                ),
            ) = inspection.cause()
            else {
                panic!(
                    "entry must name the capture reservation: {:?}",
                    inspection.cause()
                );
            };
            let pressure = allocation
                .pressure()
                .expect("native Recovery scope pressure");
            assert_eq!(pressure.scope(), Scope::Recovery);
            assert_eq!(
                pressure.requested(),
                standing,
                "the whole standing reservation"
            );
            assert_eq!(
                snapshot_family(root),
                before_media,
                "denied before any effect"
            );
            drop(inspection);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                peer.bytes(),
                "the denied entry holds nothing but the peer"
            );
            drop(peer);
            backing_census::assert_disposed("capture reservation denial", |dimension| {
                let counters = observer.snapshot().for_dimension(dimension);
                (
                    counters.active_units(),
                    counters.admitted_units(),
                    counters.released_units(),
                )
            });
        })
        .unwrap()
        .join()
        .expect("serving capture reservation worker");
}

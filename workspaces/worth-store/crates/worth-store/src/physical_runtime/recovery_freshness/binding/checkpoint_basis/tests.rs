//! Native backing protects construction, poisoned outcomes, and pool provenance.

use super::super::operations::RecoveryBindingOperationsCapacity;
use super::*;
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;
use std::num::NonZeroU64;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};

mod first_failure;
mod fixture;

const ORIGINAL: u64 = 16 * 1024;
type Denial = StoreRecoveryCheckpointBindingAllocationDenial;

#[test]
fn binding_basis_constructor_denies_native_pressure_before_storage_admission() {
    let (directory, media, mut coordination) = fixture::coordination();
    let observer = coordination.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    fixture::with_binding(
        media.store_identity(),
        Some(b"negative semantic payload"),
        |assembly, _, _| {
            let shared = coordination.admit_shared_checkpoint(assembly).unwrap();
            let retained = shared.owned_heap_bytes().unwrap();
            let requested = RecoveryBindingOperationsCapacity::for_records(1)
                .unwrap()
                .requested_bytes();
            let held = coordination
                .certification_begin_recovery_allocation(
                    NonZeroU64::new(ORIGINAL - retained - requested + 1).unwrap(),
                )
                .unwrap();
            let mut window = coordination.begin_source_read_allocation().unwrap();
            let before = observer.snapshot().for_dimension(dimension);
            let denial = window
                .begin_checkpoint_binding_rebuild(&shared, 1)
                .err()
                .unwrap();
            assert_eq!(
                denial,
                Denial::Backing {
                    requested,
                    cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                        required: ORIGINAL + 1,
                        admitted: ORIGINAL,
                    },
                }
            );
            let after = observer.snapshot().for_dimension(dimension);
            assert_eq!(after.admissions(), before.admissions());
            assert_eq!(after.admitted_units(), before.admitted_units());
            assert_eq!(after.denials(), before.denials() + 1);
            assert_eq!(window.charged_bytes(), 0);
            drop(held);
            let builder = window.begin_checkpoint_binding_rebuild(&shared, 1).unwrap();
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                retained + requested
            );
            drop(builder);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                retained
            );
        },
    );
    drop(coordination);
    drop(media);
    drop(directory);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

#[test]
fn ignored_decode_backing_refusal_keeps_finished_binding_basis_unavailable() {
    let (_directory, media, mut coordination) = fixture::coordination();
    let observer = coordination.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    // Byte-integrity-valid negative input. It must never reach semantic decode
    // under pressure; this case grants no binding or WAL authority.
    let payload = b"integrity admitted but not semantically decoded";
    fixture::with_binding(
        media.store_identity(),
        Some(payload),
        |assembly, binding, record| {
            let shared = coordination.admit_shared_checkpoint(assembly).unwrap();
            let facts = shared.facts();
            let retained = shared.owned_heap_bytes().unwrap();
            let backing = RecoveryBindingOperationsCapacity::for_records(1)
                .unwrap()
                .requested_bytes();
            let requested = checkpoint_binding_decode_peak(payload.len()).unwrap();
            let held = coordination
                .certification_begin_recovery_allocation(
                    NonZeroU64::new(ORIGINAL - retained - backing - requested + 1).unwrap(),
                )
                .unwrap();
            let mut window = coordination.begin_source_read_allocation().unwrap();
            let mut rebuilder = window.begin_checkpoint_binding_rebuild(&shared, 1).unwrap();
            let before = observer.snapshot().for_dimension(dimension);
            assert_eq!(
                rebuilder.consume(binding.unwrap(), record.unwrap(), &mut window),
                Err(Denial::Backing {
                    requested,
                    cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                        required: ORIGINAL + 1,
                        admitted: ORIGINAL,
                    },
                })
            );
            let after = observer.snapshot().for_dimension(dimension);
            assert_eq!(after.admissions(), before.admissions());
            assert_eq!(after.admitted_units(), before.admitted_units());
            assert_eq!(after.denials(), before.denials() + 1);
            assert_eq!(window.charged_bytes(), 0);
            // Ignore consume's Err deliberately. Exact C9 aggregate equality makes
            // poisoning, rather than a mismatched footer, the rejection boundary.
            let summary = rebuilder.aggregate.summary();
            assert_eq!(
                summary.record_count(),
                facts.footer().binding_record_count()
            );
            assert_eq!(
                summary.encoded_bytes(),
                facts.footer().binding_record_bytes()
            );
            assert_eq!(summary.digest(), facts.footer().binding_records_digest());
            let basis = rebuilder.finish().unwrap();
            assert!(basis.matches_checkpoint(&facts));
            assert_eq!(
                basis.evidence(&facts, 1).unwrap_err().denial(),
                StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding
            );
            assert_eq!(basis.owned_heap_bytes(), Some(0));
            assert_eq!(basis.charged_bytes(), 0);
            drop(held);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                retained
            );
            drop(basis);
        },
    );
    drop(coordination);
    drop(media);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

#[test]
fn empty_basis_keeps_real_pool_origin_and_rejects_same_store_foreign_pool() {
    let (directory, media, mut coordination) = fixture::coordination();
    let observer = coordination.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let shared = fixture::with_binding(media.store_identity(), None, |assembly, _, _| {
        coordination.admit_shared_checkpoint(assembly).unwrap()
    });
    let nonempty_shared = fixture::with_binding(
        media.store_identity(),
        Some(b"negative semantic payload"),
        |assembly, _, _| coordination.admit_shared_checkpoint(assembly).unwrap(),
    );
    let facts = shared.facts();
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    let basis = window
        .begin_checkpoint_binding_rebuild(&shared, 0)
        .unwrap()
        .finish()
        .unwrap();
    assert_eq!(basis.owned_heap_bytes(), Some(0));
    assert_eq!(basis.charged_bytes(), 0);
    assert!(basis.backing.matches_window(&window));
    assert!(basis.evidence(&facts, 0).unwrap().is_empty());
    assert_eq!(observer.snapshot().for_dimension(dimension), before);
    drop(window);
    let store = media.store_identity();
    let old_retained =
        shared.owned_heap_bytes().unwrap() + nonempty_shared.owned_heap_bytes().unwrap();
    // Release the actual exclusive recovery filesystem owner before qualifying
    // its namespace again. Shared backing and inline basis provenance survive
    // owner disposal, so the next legitimate owner has a genuinely new pool.
    drop(coordination);
    drop(media);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        old_retained
    );
    let (other_media, mut other) = fixture::reopen(directory.path());
    assert_eq!(other_media.store_identity(), store);
    let other_observer = other.certification_residency_allocations();
    let mut other_window = other.begin_source_read_allocation().unwrap();
    assert!(!basis.backing.matches_window(&other_window));
    let before = other_observer.snapshot();
    assert_eq!(
        other_window
            .begin_checkpoint_binding_rebuild(&shared, 0)
            .err()
            .unwrap(),
        Denial::PoolMismatch
    );
    assert_eq!(
        other_window
            .begin_checkpoint_binding_rebuild(&nonempty_shared, 1)
            .err()
            .unwrap(),
        Denial::PoolMismatch
    );
    assert_eq!(other_observer.snapshot(), before);
    drop(other_window);
    drop(other);
    drop(other_media);
    drop(basis);
    drop(nonempty_shared);
    drop(shared);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        0
    );
    assert_eq!(
        other_observer
            .snapshot()
            .for_dimension(Dimension::TotalBytes)
            .active_units(),
        0
    );
}

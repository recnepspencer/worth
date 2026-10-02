//! Actual C4/C9 reread, not a semantic WAL transaction or Serving-seal fixture.
use super::*;
use crate::physical_runtime::{
    PhysicalRecoveryObservationAllocationDenial, PhysicalRecoveryRejoinResidentBoundary,
    RecoveryWalArtifactView, RecoveryWalReadFailureView,
};
use std::{mem::size_of, num::NonZeroU64};
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};

#[test]
fn final_reread_native_denial_preserves_initial_inventory_and_retry_moves_fingerprint() {
    let (root, media, coordination) = fixture::coordination();
    let (path, encoded) = fixture::wal(root.path());
    let (owner, _, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let mut first_discovery = media.bounded_discovery(64, MAX_WAL_BYTES).unwrap();
    let first = admit_complete_inventory(&mut first_discovery, &coordination).unwrap();
    assert_eq!(first.frames.len(), 1);
    let first_bytes = (first.frames.capacity() * size_of::<IntegrityAdmittedRecoveryWalFrame>()
        + first.artifacts.capacity() * size_of::<WalArtifactFingerprint>())
        as u64
        + first
            .frames
            .iter()
            .map(|frame| frame.charged_bytes())
            .sum::<u64>();
    assert_eq!(fixture::active(&ports), first_bytes);
    let media = first_discovery.finish();
    let mut discovery = media.bounded_discovery(64, MAX_WAL_BYTES).unwrap();
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(fixture::ORIGINAL - first_bytes - 1).unwrap(),
        )
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    let denial = match admit_complete_inventory(&mut discovery, &coordination) {
        Err(denial) => {
            denial.at_resident_boundary(PhysicalRecoveryRejoinResidentBoundary::FinalWalAdmission)
        }
        Ok(_) => panic!("only one byte remains for actual final reread backing"),
    };
    let Denial::WalRead {
        boundary: Some(PhysicalRecoveryRejoinResidentBoundary::FinalWalAdmission),
        cause,
    } = denial
    else {
        panic!("final reread must retain its exact allocation failure");
    };
    let RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalDirectory,
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::Residency(ResidentDenial::BudgetExceeded {
                required,
                admitted,
            }),
        ..
    } = cause.diagnostic()
    else {
        panic!("the first diagnostic owner admission must deny before C4 reads");
    };
    assert!(requested > 1);
    assert_eq!(
        (*required, *admitted),
        (fixture::ORIGINAL - 1 + requested as u64, fixture::ORIGINAL)
    );
    assert_eq!(cause.charged_bytes(), 0);
    assert_eq!(discovery.counters().wal_bytes_read, 0);
    assert_eq!(discovery.counters().bytes_read, 0);
    let after = observer.snapshot().for_dimension(dimension);
    assert_eq!(after.admissions(), before.admissions());
    assert_eq!(after.admitted_units(), before.admitted_units());
    assert_eq!(after.denials(), before.denials() + 1);
    assert_eq!(fixture::active(&ports), first_bytes + held.bytes());
    assert_eq!(std::fs::read(&path).unwrap(), encoded);
    drop(held);
    drop(cause);
    let final_read = admit_complete_inventory(&mut discovery, &coordination).unwrap();
    assert!(first.matches_reread(&final_read));
    assert_eq!(discovery.counters().wal_bytes_read, encoded.len() as u64);
    let fingerprint_bytes =
        (final_read.artifacts.capacity() * size_of::<WalArtifactFingerprint>()) as u64;
    let pointer = final_read.artifacts.as_ptr();
    drop(first);
    let fingerprint = final_read.into_fingerprint();
    assert_eq!(fingerprint.artifacts.as_ptr(), pointer);
    assert_eq!(fingerprint.charged_bytes(), fingerprint_bytes);
    assert_eq!(fixture::active(&ports), fingerprint_bytes);
    assert_eq!(std::fs::read(&path).unwrap(), encoded);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
    drop(coordination);
    assert_eq!(fixture::active(&ports), fingerprint_bytes);
    drop(fingerprint);
    assert_eq!(fixture::active(&ports), 0);
    fixture::assert_balanced(&observer);
}

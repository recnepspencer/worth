//! Native canonical scratch is denied while actual source buffers remain live.

use super::*;
use crate::entry::PhysicalRecoveryWalInventoryAllocationBoundary;
use crate::orchestration::discovery::wal::{
    discover_wal_inventory, WalDiscoveryInventoryDenialKind,
};
use worth_store::physical_runtime::{
    recovery_wal::WalSegmentArtifactIdentity, BoundedRecoveryFilesystemDiscovery,
    RecoveryWalAllocationDenial,
};

pub(super) fn deny_then_retry(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &mut RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
) {
    let observer = coordination.owner().certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let before_sources = observer.snapshot().for_dimension(dimension).active_units();
    let raw = coordination
        .owner_mut()
        .begin_source_read_allocation()
        .unwrap()
        .read_wal_payloads(
            discovery,
            limits.wal_segments(),
            RecoveryReadBudget::declared(
                &limits.declaration(),
                crate::entry::PhysicalRecoveryLimitDimension::WalBytes,
            )
            .grant(),
        )
        .unwrap();
    let source_bytes = raw
        .artifacts()
        .iter()
        .map(|artifact| artifact.bytes().map_or(0, |bytes| bytes.len() as u64))
        .sum::<u64>();
    assert!(source_bytes > 0);
    let requested = (raw.artifacts().len()
        * std::mem::size_of::<(WalSegmentArtifactIdentity, &ObservedWalArtifact)>())
        as u64;
    let with_sources = observer.snapshot().for_dimension(dimension).active_units();
    assert_eq!(with_sources, before_sources + raw.charged_bytes());
    let held = coordination
        .owner()
        .certification_begin_recovery_allocation(
            NonZeroU64::new(ORIGINAL - with_sources - requested + 1).unwrap(),
        )
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    let read_before = discovery.counters().wal_bytes_read;
    let failure = match discover_wal_inventory(
        coordination.owner(),
        raw.artifacts(),
        source_bytes,
        discovery.store_identity(),
        limits.declaration().wal_frames,
    ) {
        Ok(_) => panic!("canonical slots must deny before C9 admission"),
        Err(failure) => failure,
    };
    let WalDiscoveryInventoryDenialKind::InventoryAllocation {
        boundary: PhysicalRecoveryWalInventoryAllocationBoundary::CanonicalOrdering,
        cause:
            RecoveryWalAllocationDenial::Backing {
                requested: actual,
                cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            },
    } = failure.kind
    else {
        panic!("the canonical native grant must be the failing boundary");
    };
    assert_eq!(actual, requested);
    assert_eq!(required, ORIGINAL + 1);
    assert_eq!(admitted, ORIGINAL);
    assert_eq!(failure.inventory.frames_scanned, 0);
    assert_eq!(failure.inventory.ingress.owner_projection_entries, 0);
    assert!(failure.inventory.observations.is_empty());
    assert_eq!(discovery.counters().wal_bytes_read, read_before);
    let after = observer.snapshot().for_dimension(dimension);
    assert_eq!(after.admissions(), before.admissions());
    assert_eq!(after.admitted_units(), before.admitted_units());
    assert_eq!(after.active_units(), before.active_units());
    assert_eq!(after.denials(), before.denials() + 1);
    drop(failure.inventory);
    drop(held);
    let mut healthy = match discover_wal_inventory(
        coordination.owner(),
        raw.artifacts(),
        source_bytes,
        discovery.store_identity(),
        limits.declaration().wal_frames,
    ) {
        Ok(healthy) => healthy,
        Err(_) => panic!("same real source inputs must retry after pressure is released"),
    };
    assert!(healthy.valid_frames > 0);
    assert_eq!(healthy.observed_bytes, source_bytes);
    super::selection_pressure::deny_then_retry(
        discovery,
        coordination,
        limits,
        raw.artifacts(),
        &mut healthy,
    );
    drop(healthy);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        with_sources
    );
    drop(raw);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        before_sources
    );
}

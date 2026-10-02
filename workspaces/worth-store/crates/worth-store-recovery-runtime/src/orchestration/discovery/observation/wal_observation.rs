//! Read and classify WAL sources while their raw payload reservation remains live.

use super::super::wal::{discover_wal_inventory, WalDiscoveryInventoryDenialKind};
use super::super::{discovery_limit, DiscoveryFailure, WalDiscovery};
use super::counters::record_wal_counters;
use crate::entry::{
    PhysicalRecoveryBlockKind as PhysicalRecoveryBlock, PhysicalRecoveryLimitDimension,
    PhysicalRecoveryLimits,
};
use crate::progression::PhysicalRecoveryDiscoveryCounters;
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_recovery_physics::PhysicalRecoveryResidue;

mod allocation;

pub(super) fn observe_wal(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &mut crate::orchestration::RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<(WalDiscovery, Vec<PhysicalRecoveryResidue>, u64), DiscoveryFailure> {
    let declaration = limits.declaration();
    let observed = {
        let mut allocation = coordination
            .owner_mut()
            .begin_source_read_allocation()
            .map_err(allocation::window_admission_failure)?;
        allocation
            .read_wal_payloads(discovery, declaration.wal_segments, declaration.wal_bytes)
            .map_err(allocation::map_read_failure)?
    };
    let wal_entries = observed.artifacts().len() as u64;
    let inspected = match discover_wal_inventory(
        coordination.owner(),
        observed.artifacts(),
        discovery.store_identity(),
        declaration.wal_frames,
    ) {
        Ok(inspected) => inspected,
        Err(denial) => {
            let (wal, residue) = finish_wal_inventory(denial.inventory);
            record_wal_counters(counters, &wal, &residue, wal_entries);
            let observations = wal.integrity_observations;
            let failure = match denial.kind {
                WalDiscoveryInventoryDenialKind::CounterOverflow => {
                    DiscoveryFailure::from(PhysicalRecoveryBlock::DiscoveryLimit)
                }
                WalDiscoveryInventoryDenialKind::FrameLimitExceeded { observed, admitted } => {
                    discovery_limit(
                        PhysicalRecoveryLimitDimension::WalFrames,
                        observed,
                        admitted,
                    )
                }
                WalDiscoveryInventoryDenialKind::SourceBinding => {
                    DiscoveryFailure::from(PhysicalRecoveryBlock::WalInventory)
                }
                WalDiscoveryInventoryDenialKind::Allocation(cause) => {
                    allocation::admission_failure(cause)
                }
                WalDiscoveryInventoryDenialKind::InventoryAllocation { boundary, cause } => {
                    allocation::inventory_failure(boundary, cause)
                }
            };
            return Err(failure.with_integrity_observations(observations));
        }
    };
    if inspected.observed_bytes > declaration.wal_bytes {
        let observed_bytes = inspected.observed_bytes;
        let (wal, residue) = finish_wal_inventory(inspected);
        record_wal_counters(counters, &wal, &residue, wal_entries);
        return Err(discovery_limit(
            PhysicalRecoveryLimitDimension::WalBytes,
            observed_bytes,
            declaration.wal_bytes,
        )
        .with_integrity_observations(wal.integrity_observations));
    }
    debug_assert!(inspected.frames_scanned <= declaration.wal_frames);
    let (wal, residue) = finish_wal_inventory(inspected);
    Ok((wal, residue, wal_entries))
}

fn finish_wal_inventory(
    inspected: super::super::wal::WalDiscoveryInventory,
) -> (WalDiscovery, Vec<PhysicalRecoveryResidue>) {
    let residue = inspected.residue;
    let valid_segments = inspected.candidates.len() as u64;
    let wal = WalDiscovery {
        rejected: !inspected.corruptions.is_empty(),
        candidates: inspected.candidates,
        admitted: inspected.admitted,
        integrity_observations: inspected.observations.finish(),
        integrity_ingress: inspected.ingress,
        scanned_frames: inspected.frames_scanned,
        valid_frames: inspected.valid_frames,
        valid_bytes: inspected.valid_bytes,
        observed_bytes: inspected.observed_bytes,
        torn_suffix_frames: inspected.torn_suffix_frames,
        torn_suffix_bytes: inspected.torn_suffix_bytes,
        corruption_denials: inspected.corruptions.len() as u64,
        scanned_segments: inspected.canonical_segments,
        valid_segments,
        corruptions: inspected.corruptions,
    };
    (wal, residue)
}

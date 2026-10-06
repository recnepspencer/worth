use worth_store::physical_runtime::{
    recovery_wal::WalSegmentArtifactIdentity, ObservedWalArtifact,
};
use worth_store_physical_format::store_namespace::{NamespaceEntryType, StableStoreIdentity};
use worth_store_recovery_physics::{PhysicalRecoveryResidue, PhysicalRecoveryResidueKind};

use crate::entry::{
    PhysicalRecoveryWalIntegrityDenial, PhysicalRecoveryWalInventoryAllocationBoundary as Boundary,
    WalIntegrityObservationBuilder,
};
use crate::integrity_ingress::RecoveryIntegrityIngressCounters;
use crate::orchestration::NativeWalRoster;

mod admission;
mod admitted_inventory;
mod conclusion;
mod inventory_accumulation;
pub(super) mod observation_projection;

pub(crate) use admitted_inventory::AdmittedWalInventory;

pub(super) struct WalDiscoveryInventory {
    pub candidates: crate::orchestration::wal_selection::ResidentWalCandidates,
    pub admitted: AdmittedWalInventory,
    pub residue: Vec<PhysicalRecoveryResidue>,
    pub corruptions: Vec<PhysicalRecoveryWalIntegrityDenial>,
    pub observations: WalIntegrityObservationBuilder,
    pub ingress: RecoveryIntegrityIngressCounters,
    pub canonical_segments: u64,
    pub frames_scanned: u64,
    pub valid_frames: u64,
    pub valid_bytes: u64,
    pub observed_bytes: u64,
    pub torn_suffix_frames: u64,
    pub torn_suffix_bytes: u64,
}

pub(super) enum WalDiscoveryInventoryDenialKind {
    CounterOverflow,
    /// The frames needed past those the refusing count was handed: a
    /// segment's share of what remained, or the whole WAL's frames.
    FrameLimitExceeded {
        observed: u64,
        admitted: u64,
    },
    SourceBinding,
    Allocation(worth_store::physical_runtime::RecoveryWalAllocationDenial),
    InventoryAllocation {
        boundary: Boundary,
        cause: worth_store::physical_runtime::RecoveryWalAllocationDenial,
    },
}

pub(super) struct WalDiscoveryInventoryDenial {
    pub kind: WalDiscoveryInventoryDenialKind,
    pub inventory: WalDiscoveryInventory,
}

/// Classifies the WAL files `observed` read, which returned `observed_bytes`
/// in all: what recovery's WAL-bytes budget charged for them.
pub(super) fn discover_wal_inventory(
    owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
    observed: &[ObservedWalArtifact],
    observed_bytes: u64,
    store: StableStoreIdentity,
    maximum_frames: u64,
) -> Result<WalDiscoveryInventory, WalDiscoveryInventoryDenial> {
    let (mut canonical, mut residue) =
        partition_entries(owner, observed).map_err(|cause| WalDiscoveryInventoryDenial {
            kind: WalDiscoveryInventoryDenialKind::InventoryAllocation {
                boundary: Boundary::CanonicalOrdering,
                cause,
            },
            inventory: WalDiscoveryInventory::new(0, observed_bytes, Vec::new()),
        })?;
    canonical
        .as_mut_slice()
        .sort_unstable_by_key(|(identity, _)| *identity);
    let canonical_count = canonical.len();
    let mut inventory = WalDiscoveryInventory::new(
        canonical_count as u64,
        observed_bytes,
        std::mem::take(&mut residue),
    );
    inventory.admitted = match AdmittedWalInventory::prepare(owner, canonical_count) {
        Ok(admitted) => admitted,
        Err(cause) => {
            return Err(
                inventory.deny(WalDiscoveryInventoryDenialKind::InventoryAllocation {
                    boundary: Boundary::AdmittedSegments,
                    cause,
                }),
            )
        }
    };
    inventory.candidates = match crate::orchestration::wal_selection::ResidentWalCandidates::prepare(
        owner,
        canonical_count,
    ) {
        Ok(candidates) => candidates,
        Err(cause) => {
            return Err(
                inventory.deny(WalDiscoveryInventoryDenialKind::InventoryAllocation {
                    boundary: Boundary::CandidateRoster,
                    cause,
                }),
            )
        }
    };
    for (index, (identity, artifact)) in canonical.as_slice().iter().copied().enumerate() {
        let Some(remaining) = maximum_frames.checked_sub(inventory.frames_scanned) else {
            return Err(terminal_denial(
                WalDiscoveryInventoryDenialKind::CounterOverflow,
                inventory,
            ));
        };
        let transcript = match admission::admit_segment(
            owner,
            identity,
            artifact,
            store,
            remaining,
            &mut inventory.observations,
        ) {
            Ok(transcript) => transcript,
            Err(failure) => return Err(inventory.deny_admission(failure)),
        };
        let policy_attempts = inventory_accumulation::policy_attempts(&transcript);
        let Some(attempted) = inventory.frames_scanned.checked_add(policy_attempts) else {
            return Err(terminal_denial(
                WalDiscoveryInventoryDenialKind::CounterOverflow,
                inventory,
            ));
        };
        if attempted > maximum_frames {
            return Err(terminal_denial(
                WalDiscoveryInventoryDenialKind::FrameLimitExceeded {
                    observed: attempted,
                    admitted: maximum_frames,
                },
                inventory,
            ));
        }
        if !inventory.record_ingress(transcript.counters) {
            return Err(inventory.deny(WalDiscoveryInventoryDenialKind::CounterOverflow));
        }
        let terminal = index + 1 == canonical_count;
        let conclusion = match conclusion::conclude_segment(
            transcript,
            terminal,
            owner,
            &mut inventory.candidates,
        ) {
            Ok(conclusion) => conclusion,
            Err(failure) => return Err(inventory.deny_conclusion(failure)),
        };
        if !inventory.record_conclusion(attempted, conclusion) {
            return Err(inventory.deny(WalDiscoveryInventoryDenialKind::CounterOverflow));
        }
    }
    Ok(inventory)
}

fn terminal_denial(
    kind: WalDiscoveryInventoryDenialKind,
    inventory: WalDiscoveryInventory,
) -> WalDiscoveryInventoryDenial {
    WalDiscoveryInventoryDenial { kind, inventory }
}

fn partition_entries<'source>(
    owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
    observed: &'source [ObservedWalArtifact],
) -> Result<
    (
        NativeWalRoster<(WalSegmentArtifactIdentity, &'source ObservedWalArtifact)>,
        Vec<PhysicalRecoveryResidue>,
    ),
    worth_store::physical_runtime::RecoveryWalAllocationDenial,
> {
    let mut canonical = NativeWalRoster::with_capacity(owner, observed.len())?;
    let mut residue = Vec::new();
    for artifact in observed {
        let name = artifact.name().to_string_lossy();
        if artifact.entry_type() != NamespaceEntryType::RegularFile {
            residue.push(PhysicalRecoveryResidue::new(
                name.into_owned(),
                PhysicalRecoveryResidueKind::NonRegularWalEntry,
            ));
        } else if let Some(identity) = WalSegmentArtifactIdentity::parse(&name) {
            canonical.push_reserved((identity, artifact));
        } else {
            residue.push(PhysicalRecoveryResidue::new(
                name.into_owned(),
                PhysicalRecoveryResidueKind::NonCanonicalWalArtifact,
            ));
        }
    }
    Ok((canonical, residue))
}

use worth_store::physical_runtime::{
    recovery_wal::WalSegmentArtifactIdentity, ObservedWalArtifact,
};
use worth_store_physical_format::store_namespace::{NamespaceEntryType, StableStoreIdentity};
use worth_store_recovery_physics::{
    PhysicalRecoveryResidue, PhysicalRecoveryResidueKind, PhysicalWalSegmentCandidate,
};

use crate::entry::{PhysicalRecoveryWalIntegrityDenial, PhysicalRecoveryWalIntegrityObservation};
use crate::integrity_ingress::RecoveryIntegrityIngressCounters;

mod admission;
mod admitted_inventory;
mod conclusion;
mod inventory_accumulation;
pub(super) mod observation_projection;

pub(crate) use admitted_inventory::AdmittedWalInventory;

pub(super) struct WalDiscoveryInventory {
    pub candidates: Vec<PhysicalWalSegmentCandidate>,
    pub admitted: AdmittedWalInventory,
    pub residue: Vec<PhysicalRecoveryResidue>,
    pub corruptions: Vec<PhysicalRecoveryWalIntegrityDenial>,
    pub observations: Vec<PhysicalRecoveryWalIntegrityObservation>,
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
    FrameLimitExceeded { observed: u64, admitted: u64 },
    SourceBinding,
}

pub(super) struct WalDiscoveryInventoryDenial {
    pub kind: WalDiscoveryInventoryDenialKind,
    pub inventory: WalDiscoveryInventory,
}

pub(super) fn discover_wal_inventory(
    owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
    observed: Vec<ObservedWalArtifact>,
    store: StableStoreIdentity,
    maximum_frames: u64,
) -> Result<WalDiscoveryInventory, WalDiscoveryInventoryDenial> {
    let observed_bytes = observed
        .iter()
        .try_fold(0_u64, |total, artifact| {
            total.checked_add(artifact.bytes().map_or(0, |bytes| bytes.len() as u64))
        })
        .ok_or_else(|| WalDiscoveryInventoryDenial {
            kind: WalDiscoveryInventoryDenialKind::CounterOverflow,
            inventory: WalDiscoveryInventory::new(0, 0, Vec::new()),
        })?;
    let (mut canonical, mut residue) = partition_entries(observed);
    canonical.sort_unstable_by_key(|(identity, _)| *identity);
    let canonical_count = canonical.len();
    let mut inventory = WalDiscoveryInventory::new(
        canonical_count as u64,
        observed_bytes,
        std::mem::take(&mut residue),
    );
    for (index, (identity, artifact)) in canonical.into_iter().enumerate() {
        let Some(remaining) = maximum_frames.checked_sub(inventory.frames_scanned) else {
            return Err(terminal_denial(
                WalDiscoveryInventoryDenialKind::CounterOverflow,
                inventory,
            ));
        };
        let transcript = match admission::admit_segment(owner, identity, artifact, store, remaining)
        {
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
        let conclusion = match conclusion::conclude_segment(owner, transcript, terminal) {
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

fn partition_entries(
    observed: Vec<ObservedWalArtifact>,
) -> (
    Vec<(WalSegmentArtifactIdentity, ObservedWalArtifact)>,
    Vec<PhysicalRecoveryResidue>,
) {
    let mut canonical = Vec::new();
    let mut residue = Vec::new();
    for artifact in observed {
        let name = artifact.name().to_string_lossy().into_owned();
        if artifact.entry_type() != NamespaceEntryType::RegularFile {
            residue.push(PhysicalRecoveryResidue::new(
                name,
                PhysicalRecoveryResidueKind::NonRegularWalEntry,
            ));
        } else if let Some(identity) = WalSegmentArtifactIdentity::parse(&name) {
            canonical.push((identity, artifact));
        } else {
            residue.push(PhysicalRecoveryResidue::new(
                name,
                PhysicalRecoveryResidueKind::NonCanonicalWalArtifact,
            ));
        }
    }
    (canonical, residue)
}

//! Ordinary reopen's own checkpoint custody (the clean case): the selected
//! tag-7 certificate must bind the loaded root and the checkpoint source.
//! Durability reopen later joins this candidate to the retained WAL, whose
//! released-drop members alone still require the C.8 handoff.

use sha2::{Digest, Sha256};
use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_format::{DurablePhysicalRootManifest, TierEpochCheckpointCertificateV1};

use super::super::bootstrap::BootstrapCatalogReadLimits;
use super::super::open::{load_root_manifest, CurrentRootAdmission};
use crate::physical_runtime::{
    durability::{
        select_no_release_marker, CheckpointCustodyCandidate, CleanReopenCheckpointCustody,
        ReopenedPhysicalBindingCompaction,
    },
    record_serving::{residency::frame_ports::RecordFramePorts, RecordServingState},
    MediaOwnedPhysicalRuntime, PhysicalBindingCompactionReopenFailure,
};

type CheckpointRead =
    Result<ReopenedPhysicalBindingCompaction, PhysicalBindingCompactionReopenFailure>;

/// The media open reads beyond the loaded state: one more root manifest
/// when the checkpoint source is older than the previous root.
pub(super) struct SourceRootMedia<'a> {
    pub(super) runtime: &'a MediaOwnedPhysicalRuntime,
    pub(super) ports: &'a RecordFramePorts,
    pub(super) allocation: &'a OperationAllocationGrant,
}

pub(super) fn prepare(
    recovered: bool,
    checkpoint: &CheckpointRead,
    state: &RecordServingState,
    media: &SourceRootMedia<'_>,
) -> CheckpointCustodyCandidate {
    // A release custody head never opens without C.8: the recovered custody
    // gate denies it before Serving, so only recovery decides here.
    if recovered {
        return CheckpointCustodyCandidate::ReopenRequiresC8;
    }
    let clean = match checkpoint {
        Err(_) => None,
        Ok(ReopenedPhysicalBindingCompaction::GenerationZero) => {
            genesis(&state.current_root, state.free_space.tier_epoch_start())
        }
        Ok(ReopenedPhysicalBindingCompaction::NamespaceDurable(reopened)) => {
            selected(reopened, state, media)
        }
    };
    clean.map_or(
        CheckpointCustodyCandidate::ReopenRequiresC8,
        CheckpointCustodyCandidate::CleanReopen,
    )
}

/// A never-checkpointed root without any tier anchor or tier start may open
/// on trusted genesis custody; the retained WAL decides whether it does.
fn genesis(
    root: &DurablePhysicalRootManifest,
    tier_epoch_start: Option<u64>,
) -> Option<CleanReopenCheckpointCustody> {
    (root.tier_epoch_anchor().is_none() && tier_epoch_start.is_none()).then_some(
        CleanReopenCheckpointCustody::TrustedGenesis {
            first_root: root.generation() == 1,
        },
    )
}

fn selected(
    reopened: &crate::physical_runtime::durability::NamespaceDurablePhysicalBindingCompactionReopen,
    state: &RecordServingState,
    media: &SourceRootMedia<'_>,
) -> Option<CleanReopenCheckpointCustody> {
    let source = reopened.source();
    let source_root = source_root(state, media, source.root().generation())?;
    if source_root.tree_identity() != source.root().tree_identity() {
        return None;
    }
    let source_root_sha256: [u8; 32] =
        Sha256::digest(source_root.encode(state.format.declaration())).into();
    let anchor = state.current_root.tier_epoch_anchor();
    let selected = select_no_release_marker(
        reopened.certificate_records().iter().map(AsRef::as_ref),
        source.identity(),
        source.root().generation(),
        source_root_sha256,
        anchor.is_some(),
    )?;
    let tier = match (anchor, selected.tier_payload()) {
        (None, None) => {
            if state.free_space.tier_epoch_start().is_some() {
                return None;
            }
            None
        }
        (Some(anchor), Some(payload)) => {
            let tier = TierEpochCheckpointCertificateV1::decode(payload).ok()?;
            let intent = tier.intent();
            let binds = tier.encode() == payload
                && tier.checkpoint() == source.identity()
                && tier.root_generation() == source.root().generation()
                && tier.root_sha256() == source_root_sha256
                && tier.anchor() == anchor
                && source_root.tier_epoch_anchor() == Some(anchor)
                && intent.epoch_anchor() == anchor
                && tier.completed_frame().lsn_end_exclusive()
                    <= reopened.wal_cutoff_lsn_exclusive()
                && state.free_space.tier_epoch_start() == Some(intent.tier_epoch_start());
            if !binds {
                return None;
            }
            Some(tier)
        }
        _ => return None,
    };
    Some(CleanReopenCheckpointCustody::SelectedNoRelease {
        marker: selected.marker(),
        marker_payload_sha256: selected.marker_payload_sha256(),
        tier,
    })
}

fn source_root(
    state: &RecordServingState,
    media: &SourceRootMedia<'_>,
    generation: u64,
) -> Option<DurablePhysicalRootManifest> {
    if state.current_root.generation() == generation {
        return Some(state.current_root.clone());
    }
    if let Some(previous) = state
        .previous_root
        .as_ref()
        .filter(|previous| previous.generation() == generation)
    {
        return Some(previous.clone());
    }
    let admission = CurrentRootAdmission {
        media: media.runtime.record_serving_media(),
        loader: media.ports.loader(),
        allocation: media.allocation,
        limits: BootstrapCatalogReadLimits::for_format(state.format, state.access),
        generation,
        expected_format: state.format.declaration(),
        lifecycle: media.runtime.lifecycle_state(),
        route: crate::physical_runtime::PhysicalRootProtocolRoute::OrdinaryOpen,
        counters: media.runtime.root_protocol_counter_cells(),
        resident_integrity_counters: media.ports.resident_integrity_counter_cells(),
    };
    load_root_manifest(&admission, false)
        .ok()
        .map(|(root, _)| root)
}

#[cfg(test)]
mod tests;

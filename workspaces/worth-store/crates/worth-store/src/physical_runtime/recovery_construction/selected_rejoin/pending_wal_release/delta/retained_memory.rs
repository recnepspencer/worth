//! Retained-memory accounting for the pending-WAL release transition window:
//! every observation the Store keeps while it rewalks both rooted inventories.

use worth_store_physical_format::PersistedPhysicalRecoveryProjection;
use worth_store_recovery_physics::VerifiedPendingWalReleaseCustody;

use super::super::{controls::Controls, selection::Selection};
use super::{wal_inventory, Denial, MAX_TRANSITION_MEMORY};
use crate::physical_runtime::StoreRecoveryBindingFreshnessSample;

const DISCOVERY_HEADROOM: u64 = 64 << 20;
// `observe_claim` drops the first WAL inventory before calling this budget.
// The final inventory owns at most MAX_WAL_BYTES of encoded frames; the extra
// allowance covers 65,536 frame structs, names, map nodes and its fingerprint.
const FINAL_WAL_INVENTORY_HEADROOM: u64 = wal_inventory::MAX_WAL_BYTES + (32 << 20);

pub(in crate::physical_runtime::recovery_construction::selected_rejoin::pending_wal_release) fn retained_memory(
    selected: &Selection,
    controls: &Controls,
    claim: &VerifiedPendingWalReleaseCustody,
    projection: &PersistedPhysicalRecoveryProjection,
    sample: &StoreRecoveryBindingFreshnessSample,
    checkpoint: &crate::physical_runtime::SharedRecoveryCheckpoint,
) -> Result<u64, Denial> {
    let bytes = selected
        .retained_memory_bytes()
        .saturating_add(controls.retained_memory_bytes())
        .saturating_add(claim_memory(claim))
        .saturating_add(checkpoint.owned_heap_bytes().ok_or(Denial::BoundExceeded)?)
        .saturating_add(projection_memory(projection))
        .saturating_add(sample_memory(sample)?)
        .saturating_add(DISCOVERY_HEADROOM)
        .saturating_add(FINAL_WAL_INVENTORY_HEADROOM)
        .saturating_add(8 << 20);
    (bytes < MAX_TRANSITION_MEMORY)
        .then_some(bytes)
        .ok_or(Denial::BoundExceeded)
}

fn claim_memory(claim: &VerifiedPendingWalReleaseCustody) -> u64 {
    let batches = claim
        .selected_release()
        .map_or(0, |base| 4 * std::mem::size_of_val(base.batches()) as u64);
    let projected = claim.verified_transition().map_or(0, |transition| {
        4 * std::mem::size_of_val(transition.projected()) as u64
    });
    let historical = claim.historical_batches().iter().fold(0_u64, |sum, batch| {
        let chain = batch.chain();
        let prefix = chain
            .checkpoint_prefix()
            .map_or(0, |prefix| 4 * std::mem::size_of_val(prefix.steps()) as u64);
        sum.saturating_add(4 * std::mem::size_of_val(batch) as u64)
            .saturating_add(4 * std::mem::size_of_val(chain.ordinary_steps()) as u64)
            .saturating_add(prefix)
            .saturating_add(4 * std::mem::size_of_val(chain.first_transition().projected()) as u64)
            .saturating_add(4 * std::mem::size_of_val(batch.manifest().dropped()) as u64)
    });
    let ordered = claim.ordered_history().map_or(0, |history| {
        let edges = history.edges().iter().fold(0_u64, |sum, edge| {
            let projected = match edge {
                worth_store_recovery_physics::VerifiedOrderedRootEdge::Ordinary(_) => 0,
                worth_store_recovery_physics::VerifiedOrderedRootEdge::Released(release) => {
                    4 * std::mem::size_of_val(release.transition().projected()) as u64
                }
            };
            sum.saturating_add(4 * std::mem::size_of_val(edge) as u64)
                .saturating_add(projected)
        });
        edges.saturating_add(
            claim
                .ordered_released_batches()
                .iter()
                .fold(0_u64, |sum, batch| {
                    sum.saturating_add(4 * batch.retained_bytes())
                }),
        )
    });
    (std::mem::size_of_val(claim) as u64)
        .saturating_add(batches)
        .saturating_add(projected)
        .saturating_add(historical)
        .saturating_add(ordered)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin::pending_wal_release) fn projection_memory(
    projection: &PersistedPhysicalRecoveryProjection,
) -> u64 {
    let mut bytes = (std::mem::size_of_val(projection) as u64)
        .saturating_add(4 * std::mem::size_of_val(projection.record_identities()) as u64)
        .saturating_add(4 * std::mem::size_of_val(projection.placements()) as u64)
        .saturating_add(4 * std::mem::size_of_val(projection.segment_updates()) as u64)
        .saturating_add(4 * std::mem::size_of_val(projection.manifests()) as u64)
        .saturating_add(
            4 * std::mem::size_of_val(projection.root_state().inline_allocations()) as u64,
        );
    if let Some(frames) = projection.frames() {
        bytes = bytes.saturating_add(4 * std::mem::size_of_val(frames) as u64);
        for frame in frames {
            bytes = bytes.saturating_add(frame.bytes().len() as u64);
        }
    }
    for manifest in projection.manifests() {
        bytes = bytes.saturating_add(manifest.bytes().len() as u64);
    }
    if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
        retirement: Some(retirement),
        ..
    } = projection.operation()
    {
        bytes =
            bytes.saturating_add(4 * std::mem::size_of_val(retirement.dropped_records()) as u64);
    }
    bytes
}

fn sample_memory(sample: &StoreRecoveryBindingFreshnessSample) -> Result<u64, Denial> {
    // Preserve the transition envelope's 4x roster / 1x payload allowance,
    // using actual retained capacities rather than visible slice lengths.
    let heap = sample.owned_heap_bytes().ok_or(Denial::BoundExceeded)?;
    let roster = sample.roster_heap_bytes().ok_or(Denial::BoundExceeded)?;
    roster
        .checked_mul(3)
        .and_then(|extra| heap.checked_add(extra))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of_val(sample) as u64))
        .ok_or(Denial::BoundExceeded)
}

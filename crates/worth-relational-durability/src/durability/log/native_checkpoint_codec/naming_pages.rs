//! `CHK24` capture of per-branch persistent-naming and group pages.
//!
//! Unchanged pages from a prior checkpoint are carried by content address
//! (`O(touched)` capture); pages the destination does not hold yet are copied
//! from the prior store with full SHA-256 verification.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_execution::ExecutionImmutableBytes;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::lineage::naming_pages::CheckpointNamingState;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::{
    FrozenLineageNaming, LineageSubsystem,
};
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator as Locator;
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers;

use crate::durability::data::{
    DurabilityError, DurableCheckpoint, RecoveryFailureClass, RelationalCheckpointMember,
};

fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}

fn unavailable(cause: impl std::fmt::Debug) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::ColdStorageUnavailable,
        format!("prior naming state page unavailable: {cause:?}"),
    )
}

/// Where a capture may carry naming pages instead of emitting new bytes.
pub(crate) enum NamingPageCarry<'a> {
    /// The destination's exact members: any page it holds is carried.
    Held(&'a BTreeSet<Locator>),
    /// No membership is known: carry the prior store's pages, or nothing.
    Prior(bool),
}

/// Seals `frozen` into `CHK24` naming pages and returns the manifest
/// descriptor together with its new and carried checkpoint members.
pub(crate) fn seal_naming_state(
    frozen: &FrozenLineageNaming,
    required_branches: &[contracts::history::data::coordinates::BranchId],
    carry: NamingPageCarry<'_>,
) -> Result<(CheckpointNamingState, Vec<RelationalCheckpointMember>), DurabilityError> {
    let mut sink = CollectedMembers::default();
    let state = frozen
        .seal(required_branches, &mut sink)
        .map_err(unavailable)?;
    let written: BTreeMap<Locator, Vec<u8>> = sink.members.into_iter().collect();
    let prior_store = frozen.cold_store();
    let prior_pages: BTreeSet<Locator> = prior_store
        .map(|store| store.state().page_members().into_iter().collect())
        .unwrap_or_default();
    let page_members = state.page_members();
    let mut members = Vec::with_capacity(page_members.len());
    for locator in page_members {
        let held = match carry {
            NamingPageCarry::Held(held) => held.contains(&locator),
            NamingPageCarry::Prior(carry) => carry && prior_pages.contains(&locator),
        };
        let member = if held {
            RelationalCheckpointMember::Carried { locator }
        } else if let Some(bytes) = written.get(&locator) {
            RelationalCheckpointMember::New {
                locator,
                bytes: immutable(bytes.clone()),
            }
        } else {
            let store =
                prior_store.ok_or_else(|| corrupt("carried naming page has no cold source"))?;
            let bytes = store.read_member(locator).map_err(unavailable)?;
            if bytes.len() as u64 != locator.length
                || <[u8; 32]>::from(Sha256::digest(&bytes)) != locator.sha256
            {
                return Err(unavailable("carried naming page integrity mismatch"));
            }
            RelationalCheckpointMember::New {
                locator,
                bytes: immutable(bytes),
            }
        };
        members.push(member);
    }
    Ok((state, members))
}

/// Rebuilds and pages naming and group state from a whole-history expanded
/// [`DurableCheckpoint`] for fixture / expanded encoding.
pub(crate) fn capture_expanded_naming(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    checkpoint: &DurableCheckpoint,
) -> Result<(CheckpointNamingState, Vec<RelationalCheckpointMember>), DurabilityError> {
    let required: Vec<_> = checkpoint
        .branch_cells
        .iter()
        .filter(|cell| {
            matches!(
                cell.observation.target(),
                worth_foundational::FoundationalBranchTarget::Basis(_)
            )
        })
        .map(|cell| cell.branch_id.clone())
        .collect();
    if checkpoint.envelopes.is_empty()
        && checkpoint.lineage.nodes().is_empty()
        && required.is_empty()
    {
        return Ok((CheckpointNamingState::default(), Vec::new()));
    }
    let state = crate::durability::authority::runtime_rebuild::checkpoint_restore::prepare_lineage_for_capture(
        lane, checkpoint,
    )?;
    let lineage = LineageSubsystem::new();
    lineage.install(state);
    let frozen = lineage.freeze_naming();
    seal_naming_state(&frozen, &required, NamingPageCarry::Prior(false))
}

fn immutable(bytes: Vec<u8>) -> ExecutionImmutableBytes {
    ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into_boxed_slice()))
}

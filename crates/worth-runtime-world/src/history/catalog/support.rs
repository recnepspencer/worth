use std::sync::{Arc, Mutex, MutexGuard};

use crate::identity::{CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

use super::super::reclamation::HistoryReclamationDenial;
use super::counters::lock_counters;
use super::denial::CompositeHistoryCatalogDenial;
use super::entry::CompositeHistoryCatalogEntry;
use super::reachability::lock_index;
use super::{CompositeCommitParent, CompositeHistoryCatalogState, CompositeRuntimeWorldCommit};

pub(super) fn lock_state(
    state: &Arc<Mutex<CompositeHistoryCatalogState>>,
) -> MutexGuard<'_, CompositeHistoryCatalogState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn validate_owner(
    state: &CompositeHistoryCatalogState,
    actual: RuntimeWorldOwnerIdentity,
) -> Result<(), CompositeHistoryCatalogDenial> {
    lock_counters(&state.counters).record_owner_validation();
    if actual == state.owner {
        Ok(())
    } else {
        Err(CompositeHistoryCatalogDenial::ForeignOwner {
            expected: state.owner,
            actual,
        })
    }
}

pub(super) fn validate_parent_for_reservation(
    state: &mut CompositeHistoryCatalogState,
    parent: &CompositeCommitParent,
) -> Result<(), CompositeHistoryCatalogDenial> {
    lock_counters(&state.counters).record_parent_validation();
    let CompositeCommitParent::Ordinary(parent) = parent else {
        return Ok(());
    };
    if parent.commit().owner_identity() != state.owner {
        return Err(CompositeHistoryCatalogDenial::ForeignParent {
            expected: state.owner,
            actual: parent.commit().owner_identity(),
        });
    }
    if state
        .entries
        .get(parent.commit())
        .is_none_or(|slot| slot.get().is_none())
    {
        return Err(CompositeHistoryCatalogDenial::MissingParent(
            parent.commit().clone(),
        ));
    }
    Ok(())
}

pub(super) fn release_reservation(
    state: &mut CompositeHistoryCatalogState,
    identity: &CompositeCommitIdentity,
) {
    let Some(reservation) = state.reservations.remove(identity) else {
        return;
    };
    assert!(state
        .entries
        .remove(identity)
        .is_some_and(|slot| slot.get().is_none()));
    assert!(state.inspection_order.remove(identity));
    state.publication_revision.advance();
    lock_index(&state.reachability).release_reservation(identity);
    state.metadata.release_reservation(&reservation);
    lock_counters(&state.counters).record_metadata_release();
    if let CompositeCommitParent::Ordinary(parent) = &reservation.parent {
        let mut reachability = lock_index(&state.reachability);
        reachability.decrement_descendant_dependency(parent.commit());
    }
    if matches!(reservation.parent, CompositeCommitParent::Root) {
        state.root_reserved = false;
    }
}

fn ordinary_parent_identity(parent: &CompositeCommitParent) -> Option<CompositeCommitIdentity> {
    match parent {
        CompositeCommitParent::Root => None,
        CompositeCommitParent::Ordinary(parent) => Some(parent.commit().clone()),
    }
}

pub(super) fn prevalidate_candidate_prefix(
    state: &mut CompositeHistoryCatalogState,
    candidates: &[CompositeCommitIdentity],
) -> Result<(), HistoryReclamationDenial> {
    for candidate in candidates {
        lock_counters(&state.counters).record_candidate_validation();
        if candidate.owner_identity() != state.owner {
            return Err(HistoryReclamationDenial::ForeignCandidate {
                expected: state.owner,
                actual: candidate.owner_identity(),
            });
        }
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if candidates[..index].iter().any(|prior| prior == candidate) {
            return Err(HistoryReclamationDenial::DuplicateCandidate(
                candidate.clone(),
            ));
        }
    }
    for candidate in candidates {
        if state
            .entries
            .get(candidate)
            .is_none_or(|slot| slot.get().is_none())
        {
            return Err(HistoryReclamationDenial::UnknownCandidate(
                candidate.clone(),
            ));
        }
    }
    Ok(())
}

pub(super) fn remove_installed(
    state: &mut CompositeHistoryCatalogState,
    identity: &CompositeCommitIdentity,
) -> Arc<std::sync::OnceLock<CompositeHistoryCatalogEntry>> {
    let (entry, parent) = detach_installed(state, identity);
    // The base's parent was retired before it; nothing remains to release.
    if let Some((parent, _)) = parent {
        if state.entries.contains_key(&parent) {
            lock_index(&state.reachability).decrement_descendant_dependency(&parent);
        }
    }
    entry
}

/// Removes one installed commit whose reachability row is already released,
/// and returns its effective parent. The parent's dependency is left for the
/// caller: reclamation releases it, and retirement hands it to the child.
pub(super) fn detach_installed(
    state: &mut CompositeHistoryCatalogState,
    identity: &CompositeCommitIdentity,
) -> (
    Arc<std::sync::OnceLock<CompositeHistoryCatalogEntry>>,
    Option<(CompositeCommitIdentity, usize)>,
) {
    let entry = state
        .entries
        .remove(identity)
        .expect("prevalidated candidate remains installed during reclamation");
    assert!(state.inspection_order.remove(identity));
    state.publication_revision.advance();
    let installed = entry.get().expect("prevalidated installed slot");
    let parent = effective_parent(state, installed.commit());
    state.spliced.remove(identity);
    lock_index(&state.reachability).remove_installed(identity);
    state
        .metadata
        .release_installed(installed.metadata_charge());
    lock_counters(&state.counters).record_metadata_release();
    if state.root.as_ref() == Some(identity) {
        state.root = None;
    }
    (entry, parent)
}

/// The parent `commit` depends on and the original generations between them.
/// Retirement may have spliced commits out of that edge.
pub(super) fn effective_parent(
    state: &CompositeHistoryCatalogState,
    commit: &CompositeRuntimeWorldCommit,
) -> Option<(CompositeCommitIdentity, usize)> {
    match state.spliced.get(commit.identity()) {
        Some(spliced) => Some((spliced.parent.clone(), spliced.generations)),
        None => ordinary_parent_identity(commit.parent()).map(|parent| (parent, 1)),
    }
}

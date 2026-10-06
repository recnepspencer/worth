//! Retirement of the unprotected history behind a live commit.
//!
//! History is single-rooted and never merges, so every commit has one parent.
//! Retirement walks the kept commit's ancestry and removes each commit that
//! nothing needs. A commit is needed while it holds a direct protection, a
//! second child or reservation, or performed facts whose delivery is still
//! unconsumed. A retired base hands the root to its kept child. A retired
//! interior commit is spliced out: its kept child now depends on the retired
//! commit's parent and records how many generations the edge spans. A fork
//! point and every protected commit stay, so a long-lived sibling or an old
//! held read keeps only itself, never the history between it and the head.

use std::sync::{Arc, OnceLock};

use crate::identity::CompositeCommitIdentity;

use super::support::{detach_installed, effective_parent, lock_state, validate_owner};
use super::{lock_index, CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::{CompositeHistoryCatalogEntry, CompositeHistoryCatalogState};

/// Entries removed from the catalog. Dropping them releases their component
/// pins, so the caller drops them only after the catalog lock is gone.
pub(crate) type RetiredHistoryEntries = Vec<Arc<OnceLock<CompositeHistoryCatalogEntry>>>;

/// The installed parent a kept commit depends on after retirement spliced
/// out the commits between them, and the original generations the edge spans.
#[derive(Debug, Clone)]
pub(super) struct SplicedParent {
    pub(super) parent: CompositeCommitIdentity,
    pub(super) generations: usize,
}

impl CompositeHistoryCatalog {
    /// Retires every unprotected commit on `keep`'s ancestry, oldest first.
    pub(crate) fn retire_unprotected_history(
        &self,
        keep: &CompositeCommitIdentity,
    ) -> Result<RetiredHistoryEntries, CompositeHistoryCatalogDenial> {
        let mut state = lock_state(&self.state);
        validate_owner(&state, keep.owner_identity())?;
        let ancestry = installed_ancestry(&state, keep)?;
        let mut retired = Vec::new();
        let mut child = keep.clone();
        for commit in &ancestry {
            if !retirable(&state, commit) {
                child = commit.clone();
                continue;
            }
            let (_, child_span) = installed(&state, &child)
                .and_then(|entry| effective_parent(&state, entry.commit()))
                .expect("a walked child depends on its installed parent");
            // The child's edge is the only dependency left; it ends here.
            lock_index(&state.reachability).decrement_descendant_dependency(commit);
            let (entry, parent) = detach_installed(&mut state, commit);
            retired.push(entry);
            match parent.filter(|(parent, _)| installed(&state, parent).is_some()) {
                // The child inherits the retired commit's own parent edge.
                Some((parent, span)) => {
                    let generations = child_span.saturating_add(span);
                    state.spliced.insert(
                        child.clone(),
                        SplicedParent {
                            parent,
                            generations,
                        },
                    );
                }
                None => {
                    state.spliced.remove(&child);
                    state.root = Some(child.clone());
                }
            }
        }
        retired.reverse();
        Ok(retired)
    }
}

/// Installed ancestors of `keep`, newest first. The walk ends at the root or
/// at the base whose parent was already retired.
fn installed_ancestry(
    state: &CompositeHistoryCatalogState,
    keep: &CompositeCommitIdentity,
) -> Result<Vec<CompositeCommitIdentity>, CompositeHistoryCatalogDenial> {
    let mut entry = installed(state, keep)
        .ok_or_else(|| CompositeHistoryCatalogDenial::UnknownProtectionTarget(keep.clone()))?;
    let mut ancestry = Vec::new();
    while let Some((parent, _)) = effective_parent(state, entry.commit()) {
        let Some(parent_entry) = installed(state, &parent) else {
            break;
        };
        ancestry.push(parent);
        entry = parent_entry;
    }
    Ok(ancestry)
}

fn installed<'state>(
    state: &'state CompositeHistoryCatalogState,
    identity: &CompositeCommitIdentity,
) -> Option<&'state CompositeHistoryCatalogEntry> {
    state.entries.get(identity).and_then(|slot| slot.get())
}

/// Protections are only acquired under the catalog lock this caller holds, so
/// an unprotected commit cannot gain a protection before it is removed. The
/// walk reaches a commit through its installed child, which holds one of its
/// dependencies; a single dependency therefore means no other child or
/// reservation needs it.
fn retirable(state: &CompositeHistoryCatalogState, commit: &CompositeCommitIdentity) -> bool {
    let awaits_delivery = installed(state, commit)
        .and_then(|entry| entry.publication.as_ref())
        .is_some_and(|publication| publication.awaits_delivery());
    let reachability = lock_index(&state.reachability)
        .lookup(commit)
        .expect("an installed commit has a reachability row");
    !awaits_delivery
        && reachability.direct_protections() == 0
        && reachability.descendant_dependencies() == 1
}

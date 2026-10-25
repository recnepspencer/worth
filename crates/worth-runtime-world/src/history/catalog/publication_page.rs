//! Bounded read-only traversal of canonical performed publication envelopes.

use std::num::NonZeroUsize;
use std::ops::Bound;
use std::sync::Arc;

use crate::history::retention::{CompositeHistoryProtectionObligation, HistoryProtectionClass};
use crate::history::{CanonicalPublicationEnvelope, CompositeRuntimeWorldCommit};
use crate::identity::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity, ProductBranchIdentity,
    ProductBranchIncarnation, RuntimeWorldOwnerIdentity,
};
use crate::publication::CompositeOwnerExecutionResults;

use super::support::{lock_state, validate_owner};
use super::{lock_index, CompositeHistoryCatalog, CompositeHistoryCatalogDenial};

/// Position in one live World history catalog. A cursor grants no authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorldPublicationCursor {
    frontier: RuntimeWorldPublicationFrontier,
    last_examined: CompositeCommitIdentity,
}

/// Owner-scoped immutable frontier for a bounded reconstruction pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorldPublicationFrontier {
    owner: RuntimeWorldOwnerIdentity,
    catalog_affinity: usize,
    revision: u64,
    upper_bound: Option<CompositeCommitIdentity>,
}

/// One committed publication retained by history, independent of live heads.
/// Its exact history protection prevents reclamation while the row is held.
#[derive(Debug)]
pub struct RuntimeWorldPublicationRow {
    commit: Arc<CompositeRuntimeWorldCommit>,
    envelope: Arc<CanonicalPublicationEnvelope>,
    _protection: CompositeHistoryProtectionObligation,
}

impl RuntimeWorldPublicationRow {
    pub fn commit(&self) -> &CompositeRuntimeWorldCommit {
        &self.commit
    }
    pub fn publication_attempt(&self) -> &CompositePublicationAttemptIdentity {
        self.envelope.attempt_identity()
    }
    pub fn product_branch(&self) -> &ProductBranchIdentity {
        self.facts().movement.after().branch_identity()
    }
    pub fn product_incarnation(&self) -> ProductBranchIncarnation {
        self.facts().movement.after().lifecycle_incarnation()
    }
    pub fn component_results(&self) -> &CompositeOwnerExecutionResults {
        &self.facts().component_results
    }
    fn facts(&self) -> &crate::history::PerformedPublicationFacts {
        self.envelope
            .facts()
            .expect("page holds only committed envelopes")
    }
}

/// Each examined retained or reserved slot spends one unit of `maximum`.
/// `pending` signals that a reservation or installed envelope had not yet
/// committed during this live page; callers needing a complete image fail
/// closed or restart under their own finite work bound.
#[derive(Debug)]
pub struct RuntimeWorldPublicationPage {
    rows: Vec<RuntimeWorldPublicationRow>,
    frontier: RuntimeWorldPublicationFrontier,
    next_after: Option<RuntimeWorldPublicationCursor>,
    examined: usize,
    pending: bool,
}

impl RuntimeWorldPublicationPage {
    pub fn rows(&self) -> &[RuntimeWorldPublicationRow] {
        &self.rows
    }
    pub fn frontier(&self) -> &RuntimeWorldPublicationFrontier {
        &self.frontier
    }
    pub fn into_rows(self) -> Vec<RuntimeWorldPublicationRow> {
        self.rows
    }
    pub fn next_after(&self) -> Option<&RuntimeWorldPublicationCursor> {
        self.next_after.as_ref()
    }
    pub fn examined(&self) -> usize {
        self.examined
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
}

impl CompositeHistoryCatalog {
    pub(crate) fn performed_publication_page(
        &self,
        after: Option<&RuntimeWorldPublicationCursor>,
        maximum: NonZeroUsize,
    ) -> Result<RuntimeWorldPublicationPage, CompositeHistoryCatalogDenial> {
        let state = lock_state(&self.state);
        let affinity = Arc::as_ptr(&self.state) as usize;
        let frontier = match after {
            Some(cursor) => {
                validate_frontier(&state, affinity, &cursor.frontier)?;
                cursor.frontier.clone()
            }
            None => RuntimeWorldPublicationFrontier {
                owner: state.owner,
                catalog_affinity: affinity,
                revision: state.publication_revision.current(),
                upper_bound: state.inspection_order.last().cloned(),
            },
        };
        if frontier.revision == u64::MAX {
            return Err(CompositeHistoryCatalogDenial::InspectionFrontierChanged);
        }
        let start = after.map_or(Bound::Unbounded, |cursor| {
            Bound::Excluded(&cursor.last_examined)
        });
        let end = frontier
            .upper_bound
            .as_ref()
            .map_or(Bound::Unbounded, Bound::Included);
        let mut positions = state.inspection_order.range((start, end)).peekable();
        let mut rows = Vec::with_capacity(maximum.get().min(state.entries.len()));
        let mut examined = 0;
        let mut pending = false;
        let mut last = None;
        while examined < maximum.get() {
            let Some(identity) = positions.next() else {
                break;
            };
            examined += 1;
            last = Some(identity.clone());
            let entry = state.entries.get(identity).and_then(|slot| slot.get());
            match entry
                .and_then(|entry| entry.publication.as_ref().map(|envelope| (entry, envelope)))
            {
                Some((entry, envelope)) if envelope.facts().is_some() => {
                    lock_index(&state.reachability).increment_direct_protection(identity)?;
                    rows.push(RuntimeWorldPublicationRow {
                        commit: Arc::clone(&entry.commit),
                        envelope: Arc::clone(envelope),
                        _protection: CompositeHistoryProtectionObligation::new(
                            Arc::clone(&state.reachability),
                            identity.clone(),
                            HistoryProtectionClass::ExplicitObligation,
                        ),
                    });
                }
                Some(_) | None if entry.is_none_or(|entry| entry.publication.is_some()) => {
                    pending = true
                }
                _ => {} // Root commits have no publication envelope.
            }
        }
        let next_after = if positions.peek().is_some() {
            last.map(|last_examined| RuntimeWorldPublicationCursor {
                frontier: frontier.clone(),
                last_examined,
            })
        } else {
            None
        };
        if state.publication_revision.current() != frontier.revision {
            return Err(CompositeHistoryCatalogDenial::InspectionFrontierChanged);
        }
        Ok(RuntimeWorldPublicationPage {
            rows,
            frontier,
            next_after,
            examined,
            pending,
        })
    }

    pub(crate) fn publication_frontier_is_current(
        &self,
        frontier: &RuntimeWorldPublicationFrontier,
    ) -> Result<bool, CompositeHistoryCatalogDenial> {
        let state = lock_state(&self.state);
        validate_owner(&state, frontier.owner)?;
        if frontier.catalog_affinity != Arc::as_ptr(&self.state) as usize {
            return Err(CompositeHistoryCatalogDenial::ForeignInspectionCursor);
        }
        Ok(frontier.revision != u64::MAX
            && state.publication_revision.current() == frontier.revision)
    }
}

fn validate_frontier(
    state: &super::CompositeHistoryCatalogState,
    affinity: usize,
    frontier: &RuntimeWorldPublicationFrontier,
) -> Result<(), CompositeHistoryCatalogDenial> {
    validate_owner(state, frontier.owner)?;
    if frontier.catalog_affinity != affinity {
        return Err(CompositeHistoryCatalogDenial::ForeignInspectionCursor);
    }
    if state.publication_revision.current() != frontier.revision {
        return Err(CompositeHistoryCatalogDenial::InspectionFrontierChanged);
    }
    Ok(())
}

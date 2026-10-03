use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::identity::CompositeCommitIdentity;

use super::super::retention::{CompositeHistoryProtectionObligation, HistoryProtectionClass};
use super::support::{effective_parent, lock_state, validate_owner};
use super::{lock_index, CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::{CompositeHistoryCatalogState, CompositeRuntimeWorldCommit};

/// Bounded ancestry observation. It protects every visited commit, so neither
/// explicit reclamation nor history retirement removes a visited commit or its
/// exact component pins while the page lives.
#[derive(Debug)]
pub struct CompositeHistoryTraversal {
    commits: Vec<Arc<CompositeRuntimeWorldCommit>>,
    /// Original generations between the start and each visited commit.
    /// Retirement splices unprotected commits out, so steps may exceed one.
    generations: Vec<usize>,
    next: Option<(CompositeCommitIdentity, usize)>,
    stopped_at_retired: bool,
    _protections: Vec<CompositeHistoryProtectionObligation>,
    _catalog: CompositeHistoryCatalog,
}

impl CompositeHistoryTraversal {
    pub fn commits(&self) -> impl ExactSizeIterator<Item = &CompositeRuntimeWorldCommit> {
        self.commits.iter().map(Arc::as_ref)
    }
    pub fn next_parent(&self) -> Option<&CompositeCommitIdentity> {
        self.next.as_ref().map(|(parent, _)| parent)
    }
    pub fn is_complete(&self) -> bool {
        self.next.is_none()
    }
    pub fn visited_count(&self) -> usize {
        self.commits.len()
    }
    /// Whether the page skipped commits that retirement removed, or stopped
    /// at a retired base. Such a page is not the whole ancestry it spans.
    pub fn crossed_retired_history(&self) -> bool {
        self.stopped_at_retired
            || self
                .generations
                .iter()
                .enumerate()
                .any(|(index, generations)| *generations != index)
    }

    pub(crate) fn shared_commit(&self, index: usize) -> Option<Arc<CompositeRuntimeWorldCommit>> {
        self.commits.get(index).map(Arc::clone)
    }
    pub(crate) fn generations_to(&self, index: usize) -> Option<usize> {
        self.generations.get(index).copied()
    }
    /// Original generations this page accounts for, from its start to the
    /// next parent, or through its last commit when it reached the root.
    pub(crate) fn covered_generations(&self) -> usize {
        match &self.next {
            Some((_, generations)) => *generations,
            None => self
                .generations
                .last()
                .map_or(0, |generations| generations.saturating_add(1)),
        }
    }
}

impl CompositeHistoryCatalog {
    /// Walk one parent chain up to an explicit caller bound. Reclamation does
    /// not call this method; its reachability decision is index-local. A page
    /// that reaches a retired ancestor ends there, and continuing from it
    /// fails closed because that ancestor is no longer installed.
    pub(crate) fn trace_ancestry(
        &self,
        start: CompositeCommitIdentity,
        maximum_commits: NonZeroUsize,
    ) -> Result<CompositeHistoryTraversal, CompositeHistoryCatalogDenial> {
        self.trace_ancestry_within(start, maximum_commits, usize::MAX)
    }

    /// Like `trace_ancestry`, but the page also ends before any commit that
    /// lies `span` or more original generations behind `start`.
    pub(crate) fn trace_ancestry_within(
        &self,
        start: CompositeCommitIdentity,
        maximum_commits: NonZeroUsize,
        span: usize,
    ) -> Result<CompositeHistoryTraversal, CompositeHistoryCatalogDenial> {
        let state = lock_state(&self.state);
        validate_owner(&state, start.owner_identity())?;
        let capacity = maximum_commits.get().min(state.entries.len());
        let mut commits = Vec::with_capacity(capacity);
        let mut generations = Vec::with_capacity(capacity);
        let mut next = Some((start.clone(), 0));
        let mut stopped_at_retired = false;
        while commits.len() < maximum_commits.get() {
            let Some((current, distance)) = next.take() else {
                break;
            };
            if distance >= span {
                next = Some((current, distance));
                break;
            }
            let Some(entry) = state.entries.get(&current).and_then(|slot| slot.get()) else {
                stopped_at_retired = true;
                next = Some((current, distance));
                break;
            };
            commits.push(Arc::clone(&entry.commit));
            generations.push(distance);
            next = effective_parent(&state, entry.commit())
                .map(|(parent, step)| (parent, distance.saturating_add(step)));
        }
        if commits.is_empty() {
            return Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(
                start,
            ));
        }
        let protections = commits
            .iter()
            .map(|commit| protect_installed(&state, commit.identity()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CompositeHistoryTraversal {
            commits,
            generations,
            next,
            stopped_at_retired,
            _protections: protections,
            _catalog: self.clone(),
        })
    }
}

fn protect_installed(
    state: &CompositeHistoryCatalogState,
    identity: &CompositeCommitIdentity,
) -> Result<CompositeHistoryProtectionObligation, CompositeHistoryCatalogDenial> {
    lock_index(&state.reachability).increment_direct_protection(identity)?;
    Ok(CompositeHistoryProtectionObligation::new(
        Arc::clone(&state.reachability),
        identity.clone(),
        HistoryProtectionClass::ExplicitObligation,
    ))
}

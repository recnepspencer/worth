//! Read-only view of the canonical subscription's retained per-commit touches.
//!
//! Conditional operations select the commits that can affect them from the
//! same delivered fact keys that drive output invalidation, whichever writer
//! produced the commit. Reading takes only the owner's brief cell-map section
//! and a cell image; it never enters a provider, registry, or runtime lock.

use std::collections::BTreeSet;

use worth_relational::facade::{
    history::{BranchId, CommitId},
    identity::EntityId,
    publication::PatchStreamPosition,
};

use super::fact_key::FactPostingKey;
use super::owner::SourceInvalidationOwner;

/// The records a reader watches. Conditional source records are entities;
/// `every_commit` covers whole-graph dependencies and bootstrap catch-up.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct CommitTouchInterest<'a> {
    pub(in crate::domain_computation::primary_graph) entities: &'a BTreeSet<EntityId>,
    pub(in crate::domain_computation::primary_graph) every_commit: bool,
}

/// One relevant commit on the branch and the watched entities it touched.
pub(in crate::domain_computation::primary_graph) struct TouchedCommit {
    pub(in crate::domain_computation::primary_graph) position: PatchStreamPosition,
    pub(in crate::domain_computation::primary_graph) commit: CommitId,
    /// `None` when exact touches were unavailable for the commit.
    touched: Option<BTreeSet<EntityId>>,
}

impl TouchedCommit {
    /// Unknown touches conservatively touch every watched entity.
    pub(in crate::domain_computation::primary_graph) fn touches(&self, entity: &EntityId) -> bool {
        self.touched
            .as_ref()
            .is_none_or(|touched| touched.contains(entity))
    }
}

pub(in crate::domain_computation::primary_graph) struct TouchedCommits {
    pub(in crate::domain_computation::primary_graph) commits: Vec<TouchedCommit>,
    pub(in crate::domain_computation::primary_graph) next_cursor: Option<PatchStreamPosition>,
    pub(in crate::domain_computation::primary_graph) work_remaining: bool,
    pub(in crate::domain_computation::primary_graph) caught_up_to_latest: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum CommitTouchesStop {
    /// Commits after the cursor are no longer retained. The reader must
    /// reconstruct from truth at the selected ceiling instead of skipping
    /// them, then resume after `resume_after`, the newest retained position
    /// that the ceiling already contains.
    RequiresReconstruction {
        resume_after: Option<PatchStreamPosition>,
    },
}

impl SourceInvalidationOwner {
    /// Latest delivered position across branches. Positions are globally
    /// ordered, so a cursor taken here precedes every later commit anywhere.
    pub(in crate::domain_computation::primary_graph) fn latest_position(
        &self,
    ) -> Option<PatchStreamPosition> {
        let cells = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .cells
            .values()
            .cloned()
            .collect::<Vec<_>>();
        cells
            .iter()
            .filter_map(|cell| cell.read_image().position())
            .max()
    }

    /// Commits on `branch` after `cursor`, up to the selected `ceiling`, that
    /// touch the interest. Irrelevant commits advance the cursor silently; at
    /// most `limit` relevant commits are returned per call.
    pub(in crate::domain_computation::primary_graph) fn touched_commits_after(
        &self,
        branch: &BranchId,
        cursor: Option<PatchStreamPosition>,
        ceiling: Option<CommitId>,
        limit: usize,
        interest: CommitTouchInterest<'_>,
    ) -> Result<TouchedCommits, CommitTouchesStop> {
        let cell = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .cells
            .get(branch)
            .cloned();
        let mut batch = TouchedCommits {
            commits: Vec::new(),
            next_cursor: cursor,
            work_remaining: false,
            caught_up_to_latest: true,
        };
        let Some(cell) = cell else {
            return Ok(batch);
        };
        let image = cell.read_image();
        let past = &image.payload().past;
        let within_ceiling =
            |commit: Option<CommitId>| commit.zip(ceiling).is_some_and(|(c, max)| c <= max);
        // A full window may have dropped the step that produced its oldest
        // retained position. That step is lost exactly when it lies after the
        // cursor and within the selected ceiling.
        if let Some((oldest, row)) = past.get_min() {
            if past.len() >= self.resources.installation().maximum_retained_positions
                && *oldest > cursor
                && within_ceiling(row.commit_id)
            {
                // Each key is the position its own row's commit produced.
                let resume_after = past
                    .iter()
                    .map(|(position, row)| (*position, row.commit_id))
                    .chain(std::iter::once((image.position(), image.commit_id())))
                    .take_while(|(_, commit)| within_ceiling(*commit))
                    .last()
                    .and_then(|(position, _)| position);
                return Err(CommitTouchesStop::RequiresReconstruction { resume_after });
            }
        }
        // past[p] holds the delivery that moved the branch from p to the next
        // retained position, or to the image's own position for the last row.
        let produced = past
            .iter()
            .skip(1)
            .map(|(position, row)| (*position, row.commit_id))
            .chain(std::iter::once((image.position(), image.commit_id())));
        for ((_, row), (position, commit)) in past.iter().zip(produced) {
            let (Some(position), Some(commit)) = (position, commit) else {
                continue;
            };
            if Some(position) <= cursor {
                continue;
            }
            if !within_ceiling(Some(commit)) {
                break;
            }
            let touched = row
                .next_delivery
                .keys
                .as_deref()
                .map(|keys| watched_entities(keys, interest.entities));
            let relevant =
                interest.every_commit || touched.as_ref().is_none_or(|set| !set.is_empty());
            if relevant {
                if batch.commits.len() >= limit {
                    batch.work_remaining = true;
                    break;
                }
                batch.commits.push(TouchedCommit {
                    position,
                    commit,
                    touched,
                });
            }
            batch.next_cursor = Some(position);
        }
        batch.caught_up_to_latest = batch.next_cursor >= image.position();
        Ok(batch)
    }
}

fn watched_entities(keys: &[FactPostingKey], watched: &BTreeSet<EntityId>) -> BTreeSet<EntityId> {
    keys.iter()
        .filter_map(|key| match key {
            FactPostingKey::EntityLifecycle(entity)
            | FactPostingKey::AspectRevision { entity, .. }
            | FactPostingKey::FieldRevision { entity, .. } => Some(*entity),
            _ => None,
        })
        .filter(|entity| watched.contains(entity))
        .collect()
}

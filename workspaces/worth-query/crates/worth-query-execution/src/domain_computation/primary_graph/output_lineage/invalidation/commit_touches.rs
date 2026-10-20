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
    pub(in crate::domain_computation::primary_graph) entities: CommitTouchEntities<'a>,
    pub(in crate::domain_computation::primary_graph) every_commit: bool,
}

/// Borrowed interest uses the same canonical touch stream for sets and bounded read dependencies.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum CommitTouchEntities<'a> {
    Set(&'a BTreeSet<EntityId>),
    Slice(&'a [EntityId]),
}
impl<'a> From<&'a BTreeSet<EntityId>> for CommitTouchEntities<'a> {
    fn from(value: &'a BTreeSet<EntityId>) -> Self {
        Self::Set(value)
    }
}
impl<'a> From<&'a [EntityId]> for CommitTouchEntities<'a> {
    fn from(value: &'a [EntityId]) -> Self {
        Self::Slice(value)
    }
}
impl CommitTouchEntities<'_> {
    fn contains(&self, id: &EntityId) -> bool {
        match self {
            Self::Set(v) => v.contains(id),
            Self::Slice(v) => v.contains(id),
        }
    }
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
            .filter_map(|slot| slot.admitted())
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
            .and_then(|slot| slot.admitted());
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
        if let Some((oldest, row)) = past.get_min()
            && self
                .resources
                .installation()
                .maximum_retained_positions
                .is_some_and(|maximum| past.len() >= maximum)
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

fn watched_entities(
    keys: &[FactPostingKey],
    watched: CommitTouchEntities<'_>,
) -> BTreeSet<EntityId> {
    keys.iter()
        .flat_map(|key| match key {
            FactPostingKey::EntityLifecycle(entity)
            | FactPostingKey::AspectRevision { entity, .. }
            | FactPostingKey::FieldRevision { entity, .. } => [Some(*entity), None],
            FactPostingKey::Adjacency { anchor, .. } => [Some(*anchor), None],
            FactPostingKey::RelationMembership { from, to, .. } => [Some(*from), Some(*to)],
            _ => [None, None],
        })
        .flatten()
        .filter(|entity| watched.contains(entity))
        .collect()
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn touch_position_at(
        &self,
        branch: &BranchId,
        commit: CommitId,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<PatchStreamPosition>, worth_execution::WorkCeilingDenial> {
        let cell = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .cells
            .get(branch)
            .and_then(|slot| slot.admitted());
        let Some(cell) = cell else { return Ok(None) };
        let image = cell.read_image();
        execution.consult()?;
        if image.commit_id() == Some(commit) {
            return Ok(image.position());
        }
        for (position, row) in image.payload().past.iter() {
            execution.consult()?;
            if row.commit_id == Some(commit) {
                return Ok(*position);
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod relation_interest_tests {
    use super::*;
    use worth_relational::facade::identity::{KindId, PartitionId};
    #[test]
    fn relation_only_interest_matches_full_native_anchors_and_endpoints() {
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;
        let anchor = EntityId::new(PartitionId::new(7), 11, 2);
        let target = EntityId::new(PartitionId::new(9), 13, 4);
        let recycled_anchor = EntityId::new(PartitionId::new(7), 11, 3);
        let recycled_target = EntityId::new(PartitionId::new(9), 13, 5);
        let watched = [anchor, target, recycled_anchor, recycled_target];
        let interest = CommitTouchEntities::Slice(&watched);
        // Neither input has an entity/field revision; absent-edge attachment is sufficient.
        let adjacency = FactPostingKey::Adjacency {
            kind: KindId(17),
            anchor,
            direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        };
        assert_eq!(
            watched_entities(std::slice::from_ref(&adjacency), interest),
            BTreeSet::from([anchor])
        );
        let membership = FactPostingKey::RelationMembership {
            kind: KindId(17),
            from: anchor,
            to: target,
        };
        assert_eq!(
            watched_entities(std::slice::from_ref(&membership), interest),
            BTreeSet::from([anchor, target])
        );
        let recycled = [recycled_anchor, recycled_target];
        assert!(watched_entities(&[adjacency], CommitTouchEntities::Slice(&recycled)).is_empty());
        assert!(watched_entities(&[membership], CommitTouchEntities::Slice(&recycled)).is_empty());
    }
}

//! Shared read-only commit-touch query vocabulary for source invalidation ports.

use std::collections::BTreeSet;

use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::publication::PatchStreamPosition;

/// The records a reader watches. Conditional source records are entities;
/// `every_commit` covers whole-graph dependencies and bootstrap catch-up.
#[derive(Clone, Copy)]
pub struct CommitTouchInterest<'a> {
    pub entities: CommitTouchEntities<'a>,
    pub every_commit: bool,
}

/// Borrowed interest uses the same canonical touch stream for sets and bounded read dependencies.
#[derive(Clone, Copy)]
pub enum CommitTouchEntities<'a> {
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
    pub fn contains(&self, id: &EntityId) -> bool {
        match self {
            Self::Set(v) => v.contains(id),
            Self::Slice(v) => v.contains(id),
        }
    }
}

/// One relevant commit on the branch and the watched entities it touched.
pub struct TouchedCommit {
    pub position: PatchStreamPosition,
    pub commit: CommitId,
    /// `None` when exact touches were unavailable for the commit.
    touched: Option<BTreeSet<EntityId>>,
}

impl TouchedCommit {
    pub fn new(
        position: PatchStreamPosition,
        commit: CommitId,
        touched: Option<BTreeSet<EntityId>>,
    ) -> Self {
        Self {
            position,
            commit,
            touched,
        }
    }

    /// Unknown touches conservatively touch every watched entity.
    pub fn touches(&self, entity: &EntityId) -> bool {
        self.touched
            .as_ref()
            .is_none_or(|touched| touched.contains(entity))
    }

    #[doc(hidden)]
    pub fn commit_for_test(&self) -> &CommitId {
        &self.commit
    }
}

pub struct TouchedCommits {
    pub commits: Vec<TouchedCommit>,
    pub next_cursor: Option<PatchStreamPosition>,
    pub work_remaining: bool,
    pub caught_up_to_latest: bool,
}

impl TouchedCommits {
    #[doc(hidden)]
    pub fn caught_up_to_latest_for_test(&self) -> &bool {
        &self.caught_up_to_latest
    }

    #[doc(hidden)]
    pub fn commits_for_test(&self) -> &Vec<TouchedCommit> {
        &self.commits
    }

    #[doc(hidden)]
    pub fn next_cursor_for_test(&self) -> &Option<PatchStreamPosition> {
        &self.next_cursor
    }
}

impl<'a> CommitTouchInterest<'a> {
    #[doc(hidden)]
    pub fn new_for_test(entities: CommitTouchEntities<'a>, every_commit: bool) -> Self {
        Self {
            entities,
            every_commit,
        }
    }
}

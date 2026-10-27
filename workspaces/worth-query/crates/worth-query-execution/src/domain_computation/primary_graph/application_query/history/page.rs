use std::{num::NonZeroUsize, sync::Arc};
use worth_relational::facade::history::{
    RelationalRecordHistoryChange, RelationalRecordHistoryPage,
};

use super::cursor::RetainedWorldPage;

/// Each call visits one World parent OR a finite native chunk. New page/position
/// backing is bounded; shared captured subjects, canonical envelopes and prior
/// continuation owners are not recopied or charged as newly owned backing.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryApplicationHistoryPageLimits {
    pub maximum_entries: NonZeroUsize,
    pub maximum_source_visits: NonZeroUsize,
    pub maximum_owned_bytes: NonZeroUsize,
}

/// An empty page can be incomplete. Only `is_complete` ends the selected history.
pub struct WorthQueryApplicationHistoryPage {
    pub(super) world: Option<Arc<RetainedWorldPage>>,
    pub(super) native: Option<RelationalRecordHistoryPage>,
    pub(super) complete: bool,
    pub(super) source_visits: usize,
    pub(super) newly_owned_bytes: usize,
    pub(super) authorization_work: super::super::WorthQueryApplicationAuthorizationWorkEvidence,
}

/// Borrowed canonical metadata, without a Native cursor, retained source or payload image.
pub struct WorthQueryApplicationHistoryEntry<'page> {
    native: &'page RelationalRecordHistoryChange,
}

impl WorthQueryApplicationHistoryEntry<'_> {
    pub fn commit_id(&self) -> worth_relational::facade::history::CommitId {
        self.native.commit_id()
    }
    pub fn version_id(&self) -> worth_relational::facade::identity::VersionId {
        self.native.version_id()
    }
    pub fn branch_id(&self) -> &worth_relational::facade::history::BranchId {
        self.native.branch_id()
    }
    pub fn record(&self) -> &worth_relational::facade::transactions::RecordRef {
        self.native.record()
    }
    pub fn entity_id(&self) -> Option<worth_relational::facade::identity::EntityId> {
        match self.native.record() {
            worth_relational::facade::transactions::RecordRef::Entity(id) => Some(*id),
            worth_relational::facade::transactions::RecordRef::Relation(_) => None,
        }
    }
    pub fn relation_id(&self) -> Option<worth_relational::facade::identity::RelationId> {
        match self.native.record() {
            worth_relational::facade::transactions::RecordRef::Relation(id) => Some(*id),
            worth_relational::facade::transactions::RecordRef::Entity(_) => None,
        }
    }
    pub fn structural_change(
        &self,
    ) -> worth_relational::facade::publication::RecordStructuralChange {
        self.native.structural_change()
    }
    pub fn contains_opaque_aspect(&self) -> bool {
        self.native.contains_opaque_aspect()
    }
    pub fn changed_aspects(&self) -> impl Iterator<Item = &worth_foundational::facade::AspectKey> {
        self.native.changed_aspects()
    }
}

impl WorthQueryApplicationHistoryPage {
    pub fn entries(&self) -> impl Iterator<Item = WorthQueryApplicationHistoryEntry<'_>> {
        self.native
            .iter()
            .flat_map(RelationalRecordHistoryPage::entries)
            .map(|native| WorthQueryApplicationHistoryEntry { native })
    }
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
    pub const fn source_visits(&self) -> usize {
        self.source_visits
    }
    pub const fn newly_owned_bytes(&self) -> usize {
        self.newly_owned_bytes
    }
    pub const fn authorization_work(
        &self,
    ) -> super::super::WorthQueryApplicationAuthorizationWorkEvidence {
        self.authorization_work
    }
    pub fn product_commit(&self) -> Option<&worth_runtime_world::facade::CompositeCommitIdentity> {
        self.world
            .as_ref()?
            .page
            .commit()
            .map(|commit| commit.identity())
    }
    /// A root's imported canonical component is distinct from a World publication.
    pub fn is_root_import(&self) -> bool {
        self.world
            .as_ref()
            .and_then(|world| world.page.commit())
            .is_some_and(|commit| {
                matches!(
                    commit.parent(),
                    worth_runtime_world::facade::CompositeCommitParent::Root
                ) && commit
                    .basis()
                    .relational_basis()
                    .observation()
                    .commit_id()
                    .is_some()
            })
    }
}

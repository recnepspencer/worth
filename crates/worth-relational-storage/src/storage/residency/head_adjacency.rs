//! Current adjacency access cannot select a historical kind basis.
use crate::identity::data::KindId;
use crate::storage::partition::{
    AdjacencyIds, AdjacencyKindBasis, AdjacencySet, SparseAdjacencyTable,
};
#[derive(Clone, Copy)]
pub struct RelationalHeadAdjacencyTable<'lease>(&'lease SparseAdjacencyTable);
#[derive(Clone, Copy)]
pub struct RelationalHeadAdjacencySet<'lease>(&'lease AdjacencySet);
impl<'lease> RelationalHeadAdjacencyTable<'lease> {
    pub(super) fn new(table: &'lease SparseAdjacencyTable) -> Self {
        Self(table)
    }
    pub fn navigation_work_bound(self) -> usize {
        self.0.navigation_work_bound()
    }
    pub fn get(self, slot: usize) -> Option<RelationalHeadAdjacencySet<'lease>> {
        self.0.get(slot).map(RelationalHeadAdjacencySet)
    }
}
impl<'lease> RelationalHeadAdjacencySet<'lease> {
    pub fn navigation_work_bound(self) -> usize {
        self.0
            .selected_kind_navigation_work_bound(AdjacencyKindBasis::Current)
    }
    pub fn structural_revision_navigation_work_bound(self) -> usize {
        self.0.structural_revision_navigation_work_bound()
    }
    pub fn structural_revision(self, kind: KindId) -> Option<crate::identity::data::VersionId> {
        self.0.structural_revision(kind)
    }
    pub fn kind_ids(self, kind: KindId) -> AdjacencyIds<'lease> {
        self.0.kind_ids(AdjacencyKindBasis::Current, kind)
    }
}

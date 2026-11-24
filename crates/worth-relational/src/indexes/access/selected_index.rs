//! Immutable index handles and the resident basis used to verify their rows.
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::pinned::VisibilityProjectionViewOwnerPort as _;
use crate::history::retention::VisibilitySnapshotBasis;
use crate::{
    identity::data::VersionId,
    indexes::data::{DerivedIndexGeneration, DerivedIndexId},
    runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    snapshots::data::SnapshotHandle,
    visibility::{
        materialization::read_records::projection::VisibilityProjectionView,
        snapshot_states::ResidentVisibilitySnapshotBasis,
    },
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct PinnedIndex {
    pub definition: crate::indexes::retained_definition::RetainedDerivedIndexDefinition,
    pub generation: Option<Arc<DerivedIndexGeneration>>,
}

pub(crate) struct PinnedIndexLookup<'pin> {
    pub(crate) snapshot: &'pin SnapshotHandle,
    pub(crate) resident: &'pin ResidentVisibilitySnapshotBasis,
    pub(crate) runtime_identity: u64,
    pub(crate) current_version: VersionId,
    pub(crate) instrumentation: &'pin RuntimeInstrumentation,
    pub(crate) indexes: &'pin BTreeMap<DerivedIndexId, PinnedIndex>,
}

impl PinnedIndexLookup<'_> {
    pub(super) fn selected_index(&self, index: DerivedIndexId) -> Option<&PinnedIndex> {
        self.indexes.get(&index)
    }

    pub(super) fn project_snapshot(
        &self,
        snapshot: &SnapshotHandle,
    ) -> Option<VisibilityProjectionView> {
        (self.snapshot == snapshot).then(|| {
            VisibilityProjectionView::from_resident_basis(
                self.resident,
                self.runtime_identity,
                self.current_version,
                self.instrumentation,
            )
        })
    }

    pub(super) fn basis(&self) -> &VisibilitySnapshotBasis {
        self.resident.basis()
    }
}

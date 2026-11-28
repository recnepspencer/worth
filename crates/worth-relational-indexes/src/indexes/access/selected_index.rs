//! Immutable index handles and the resident basis used to verify their rows.
use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;
use crate::branch::history::retention::selected_snapshot::VisibilitySnapshotBasis;
use crate::branch::indexes::data::SharedDerivedIndexGeneration;
use crate::branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::branch::snapshots::data::SnapshotHandle;
use crate::branch::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use crate::branch::visibility::snapshot_states::resident_basis::ResidentVisibilitySnapshotBasis;
use std::collections::BTreeMap;
use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;
use worth_relational_contracts::internal_port::indexes::data::definition::DerivedIndexId;

#[derive(Clone, Debug)]
pub struct PinnedIndex {
    pub definition: crate::branch::indexes::retained_definition::RetainedDerivedIndexDefinition,
    pub generation: Option<SharedDerivedIndexGeneration>,
}

pub struct PinnedIndexLookup<'pin> {
    pub snapshot: &'pin SnapshotHandle,
    pub resident: &'pin ResidentVisibilitySnapshotBasis,
    pub runtime_identity: u64,
    pub current_version: VersionId,
    pub instrumentation: &'pin RuntimeInstrumentation,
    pub indexes: &'pin BTreeMap<DerivedIndexId, PinnedIndex>,
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

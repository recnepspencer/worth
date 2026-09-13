use super::super::frame_identity::frame_identity;
use super::super::*;
use super::pending::PendingProjectionBasis;
use worth_store_physical_format::{
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest, PersistedRecordIdentity,
};

pub(super) struct ProjectedMaterializationBasis {
    pub(super) frames: BTreeMap<PhysicalRedoTargetIdentity, PersistedPhysicalRecoveryFrame>,
    pub(super) placements: BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    pub(super) projected_records: BTreeSet<PersistedRecordIdentity>,
    pub(super) segment_updates: BTreeMap<(u64, u64), RecordSegmentPageManifestEntry>,
    pub(super) manifests: BTreeMap<RecordArtifactFile, PersistedPhysicalRecoveryManifest>,
    pub(super) root_states: Vec<PersistedPhysicalRecoveryRootState>,
}

pub(super) fn collect(pending: &PendingProjectionBasis<'_>) -> ProjectedMaterializationBasis {
    let mut basis = ProjectedMaterializationBasis {
        frames: BTreeMap::new(),
        placements: BTreeMap::new(),
        projected_records: BTreeSet::new(),
        segment_updates: BTreeMap::new(),
        manifests: BTreeMap::new(),
        root_states: Vec::with_capacity(pending.projections.len()),
    };
    for projection in &pending.projections {
        let materialization = projection.materialization();
        basis.root_states.push(materialization.root_state().clone());
        for frame in materialization.frames() {
            basis
                .frames
                .insert(frame_identity(frame.subject()), frame.clone());
        }
        for placement in materialization.placements() {
            basis.projected_records.insert(placement.record());
            basis.placements.insert(placement.record(), *placement);
        }
        for update in materialization.segment_updates() {
            basis.segment_updates.insert(
                (update.page_cell().segment_id().get(), update.page().get()),
                *update,
            );
        }
        for manifest in materialization.manifests() {
            basis
                .manifests
                .insert(manifest.artifact(), manifest.clone());
        }
    }
    basis
}

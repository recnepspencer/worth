use crate::replay::data::ReplayObservableSurface;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;

use super::super::derived_index_surface::{
    derived_index_surface_is_promised, DERIVED_INDEX_SURFACE,
};

pub fn promised_replay_surfaces(
    snapshot_is_authoritative: bool,
    original: &CanonicalCommitEnvelope,
    recovered: Option<&CanonicalCommitEnvelope>,
) -> Vec<ReplayObservableSurface> {
    let mut surfaces = vec![
        ReplayObservableSurface::Patch,
        ReplayObservableSurface::Diagnostics,
        ReplayObservableSurface::History,
        ReplayObservableSurface::BranchHead,
    ];
    if snapshot_is_authoritative {
        surfaces.push(ReplayObservableSurface::Snapshot);
    }
    if original.has_lineage_authority()
        || recovered.is_some_and(|envelope| envelope.has_lineage_authority())
    {
        surfaces.push(ReplayObservableSurface::Lineage);
    }
    if derived_index_surface_is_promised(original, recovered) {
        surfaces.push(DERIVED_INDEX_SURFACE);
    }
    surfaces
}

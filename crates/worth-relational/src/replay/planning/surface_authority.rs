use crate::history::data::CanonicalCommitEnvelope;
use crate::replay::data::ReplayObservableSurface;

use super::super::derived_index_surface::{
    derived_index_surface_is_promised, DERIVED_INDEX_SURFACE,
};

pub(in crate::replay) fn promised_replay_surfaces(
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
    if original.strategy_artifacts.is_some()
        || recovered.is_some_and(|envelope| envelope.strategy_artifacts.is_some())
    {
        surfaces.push(ReplayObservableSurface::Strategy);
    }
    if derived_index_surface_is_promised(original, recovered) {
        surfaces.push(DERIVED_INDEX_SURFACE);
    }
    surfaces
}

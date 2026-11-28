mod commit_closure;
mod continuity;
mod diagnostics;
mod execution;
mod lineage_authority;
mod recovery_plan;
mod snapshot_surface;
mod surface_comparison;

use crate::performance::ReplayLineageAuthorityIndexedSource;
use crate::replay::data::{DescriptorComparisonBasis, ReplayAuthorityBasisKind};
use crate::runtime::RelationalRuntime;
use crate::schema::ValidatedSchemaContinuityBundle;

pub struct ReplayAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

struct ValidatedReplayContinuityEnvelope<'a> {
    _validated_bundle: ValidatedSchemaContinuityBundle<'a>,
    transition_basis: Option<DescriptorComparisonBasis>,
    continuation_basis: Option<DescriptorComparisonBasis>,
    reconciliation_basis: Option<DescriptorComparisonBasis>,
    lineage_basis: Option<DescriptorComparisonBasis>,
}

/// The published lineage artifact replay compares against, owned because a
/// durable-log or checkpoint artifact is read out of the durability subsystem
/// rather than borrowed from it.
struct SelectedPublishedLineageAuthority {
    kind: ReplayAuthorityBasisKind,
    /// `Some` exactly when the artifact came from durable storage rather than
    /// from the envelope being audited.
    indexed_source: Option<ReplayLineageAuthorityIndexedSource>,
    artifact: crate::lineage::data::PublishedLineageArtifact,
}

impl<'runtime> ReplayAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

impl RelationalRuntime {
    pub fn replay_authority(&self) -> ReplayAuthority<'_> {
        ReplayAuthority::new(self)
    }
}

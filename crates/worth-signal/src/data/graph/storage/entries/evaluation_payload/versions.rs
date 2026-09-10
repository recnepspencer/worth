//! Version updates shared by installed storage and prepared node payloads.
use crate::data::aspect::AspectVersion;
use crate::data::node::{NodeHotData, NodeWarmData};
use crate::data::output::ChangedRegion;

pub(in crate::data::graph) fn apply_aspect_version(
    hot: &mut NodeHotData,
    warm: &mut NodeWarmData,
    version: AspectVersion,
    changed_regions: &[ChangedRegion],
) {
    warm.aspect_version_overrides
        .apply_evaluation(version, changed_regions);
    hot.aspect_version_header.set_global(version);
    hot.aspect_version_header
        .set_has_partition_overrides(warm.aspect_version_overrides.has_overrides());
}

//! Advance the distinct projections of a hierarchical scoped write.
use super::{Aspect, AspectVersion, ChangedRegion, PartitionVersionOverrides, ScopeCoverage};

pub(super) fn apply(
    overrides: &mut PartitionVersionOverrides,
    version: AspectVersion,
    changed_regions: &[ChangedRegion],
) {
    if changed_regions.is_empty() {
        overrides.set_global(version);
        return;
    }
    for region in changed_regions {
        overrides.write_version(region, version);
    }
}

impl PartitionVersionOverrides {
    pub(super) fn write_version(&mut self, region: &ChangedRegion, version: AspectVersion) {
        for depth in 1..=region.path().depth() {
            let prefix = region.path().prefix(depth).expect("validated scope prefix");
            let record = self.paths.entry(prefix).or_default();
            record.descendant_aggregate = Some(version);
            if depth == region.path().depth() {
                match region.coverage() {
                    ScopeCoverage::Exact => record.exact_write = Some(version),
                    ScopeCoverage::Subtree => record.subtree_write = Some(version),
                }
            }
        }
    }

    pub(super) fn write_aspect(&mut self, region: &ChangedRegion, aspect: Aspect, value: u64) {
        for depth in 1..=region.path().depth() {
            let prefix = region.path().prefix(depth).expect("validated scope prefix");
            let record = self.paths.entry(prefix).or_default();
            set_aspect(&mut record.descendant_aggregate, aspect, value);
            if depth == region.path().depth() {
                match region.coverage() {
                    ScopeCoverage::Exact => set_aspect(&mut record.exact_write, aspect, value),
                    ScopeCoverage::Subtree => set_aspect(&mut record.subtree_write, aspect, value),
                }
            }
        }
    }
}

fn set_aspect(version: &mut Option<AspectVersion>, aspect: Aspect, value: u64) {
    *version = Some(version.unwrap_or_default().with(aspect, value));
}

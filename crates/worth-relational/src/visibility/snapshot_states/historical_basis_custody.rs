use crate::branch::root::RelationalBranchRoot;
use std::sync::Arc;

use super::HistoricalVisibilityBasis;

impl HistoricalVisibilityBasis {
    pub(crate) fn retained_root(&self) -> Option<&Arc<RelationalBranchRoot>> {
        self.root.as_ref().map(|retained| retained.root())
    }
}

impl super::HistoricalVisibilityBasis {
    pub(crate) fn from_retained_interval(
        branch_id: crate::history::data::BranchId,
        version_id: crate::identity::data::VersionId,
        source_version: crate::identity::data::VersionId,
        root: crate::history::retention::RelationalRetainedHistoricalRoot,
    ) -> Self {
        Self {
            branch_id,
            version_id,
            coverage: super::HistoricalVisibilityCoverage::RetainedInterval {
                source_root_id: root.root().id(),
                source_version,
            },
            root: Some(std::sync::Arc::new(root)),
        }
    }
}

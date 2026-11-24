use crate::branch::root::RelationalBranchRoot;
use std::sync::Arc;

use super::HistoricalVisibilityBasis;

impl HistoricalVisibilityBasis {
    pub(crate) fn retained_root(&self) -> Option<&Arc<RelationalBranchRoot>> {
        self.root.as_ref().map(|retained| retained.root())
    }
}

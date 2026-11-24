use std::sync::Arc;

use super::RelationalBranchReferenceCell;

impl RelationalBranchReferenceCell {
    pub fn coordination(
        &self,
    ) -> &Arc<super::super::coordination::RelationalBranchCoordinationCell> {
        &self.coordination
    }

    pub fn head_retention(
        &self,
    ) -> &Arc<crate::history::retention::RelationalBranchHeadRetentionCell> {
        &self.head_retention
    }
}

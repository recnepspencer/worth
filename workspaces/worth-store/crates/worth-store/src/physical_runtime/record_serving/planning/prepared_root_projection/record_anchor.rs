use worth_store_physical_format::{RecordAnchorAdvance, RecordAnchorTable};

use super::{DerivedRootUpdates, PreparedPhysicalRootProjection};

impl PreparedPhysicalRootProjection {
    /// Binds this publication's anchor advance after it was checked against
    /// the source root, before any WAL effect.
    pub(in crate::physical_runtime::record_serving) fn set_record_anchor_advance(
        &mut self,
        advance: RecordAnchorAdvance,
    ) {
        self.derived_updates.record_anchor_advance = Some(advance);
    }

    pub(in crate::physical_runtime) const fn recovery_record_anchor_advance(
        &self,
    ) -> Option<RecordAnchorAdvance> {
        self.derived_updates.record_anchor_advance
    }
}

impl DerivedRootUpdates {
    /// The anchors the successor root selects: the current table, advanced by
    /// this publication's request. `None` means the current root no longer
    /// matches the generation checked during planning.
    pub(in crate::physical_runtime::record_serving) fn successor_record_anchors(
        &self,
        current: RecordAnchorTable,
    ) -> Option<RecordAnchorTable> {
        match self.record_anchor_advance {
            Some(advance) => current.advanced(advance).ok(),
            None => Some(current),
        }
    }
}

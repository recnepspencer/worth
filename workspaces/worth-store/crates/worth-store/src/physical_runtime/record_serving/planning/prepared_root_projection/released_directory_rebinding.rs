//! The same-member directory rebinding projected by a released Drop.

use worth_store_physical_format::{DerivedFamilyRootDirectoryBinding, PersistedRecordIdentity};

use super::PreparedPhysicalRootProjection;
use crate::physical_runtime::record_serving::PreparedReleasedDirectoryRebinding;

impl PreparedPhysicalRootProjection {
    /// The predecessor remains routed residue, not an extra authorized drop.
    pub(in crate::physical_runtime) fn set_released_directory_rebinding(
        &mut self,
        record: PersistedRecordIdentity,
        basis: PreparedReleasedDirectoryRebinding,
    ) -> Option<()> {
        if self.derived_updates.directory.is_some()
            || self.derived_updates.released_directory_rebinding.is_some()
            || self.derived_updates.expected_previous_directory.is_some()
        {
            return None;
        }
        self.derived_updates.directory = Some(DerivedFamilyRootDirectoryBinding::new(record, None));
        self.derived_updates.expected_previous_directory = Some(Some(basis.expected_previous()));
        self.derived_updates.indexed_through_quarantine = Some(basis.quarantined_through());
        self.derived_updates.released_directory_rebinding = Some(basis);
        Some(())
    }
}

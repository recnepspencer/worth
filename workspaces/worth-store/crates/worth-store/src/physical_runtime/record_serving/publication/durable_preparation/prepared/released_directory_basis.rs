//! Selected source-directory facts retained across a fenced released drop.

use worth_store_physical_format::{DerivedFamilyRootDirectoryBinding, PersistedRecordIdentity};

#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct PreparedReleasedDirectoryRebinding {
    expected_previous: DerivedFamilyRootDirectoryBinding,
    expected_previous_payload_sha256: [u8; 32],
    next_payload_sha256: [u8; 32],
    quarantined_through: Option<PersistedRecordIdentity>,
}

impl PreparedReleasedDirectoryRebinding {
    pub(in crate::physical_runtime) const fn from_selected_source(
        expected_previous: DerivedFamilyRootDirectoryBinding,
        expected_previous_payload_sha256: [u8; 32],
        next_payload_sha256: [u8; 32],
        quarantined_through: Option<PersistedRecordIdentity>,
    ) -> Self {
        Self {
            expected_previous,
            expected_previous_payload_sha256,
            next_payload_sha256,
            quarantined_through,
        }
    }

    pub(in crate::physical_runtime) const fn expected_previous(
        self,
    ) -> DerivedFamilyRootDirectoryBinding {
        self.expected_previous
    }

    pub(in crate::physical_runtime) const fn expected_previous_payload_sha256(self) -> [u8; 32] {
        self.expected_previous_payload_sha256
    }

    pub(in crate::physical_runtime) const fn next_payload_sha256(self) -> [u8; 32] {
        self.next_payload_sha256
    }

    pub(in crate::physical_runtime) const fn quarantined_through(
        self,
    ) -> Option<PersistedRecordIdentity> {
        self.quarantined_through
    }
}

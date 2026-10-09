use crate::DerivedFamilyRootDirectoryBinding;

use super::PersistedDerivedDirectoryRecordBinding;

/// Wire observation of the selected directory replaced by a released drop.
/// The old digest and the new binding are not authority until C.9 joins both
/// to selected source bytes and the admitted second WAL record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedReleasedDirectoryReplacementV1 {
    expected_previous: DerivedFamilyRootDirectoryBinding,
    expected_previous_payload_sha256: [u8; 32],
    next: PersistedDerivedDirectoryRecordBinding,
}

impl PersistedReleasedDirectoryReplacementV1 {
    pub fn new(
        expected_previous: DerivedFamilyRootDirectoryBinding,
        expected_previous_payload_sha256: [u8; 32],
        next: PersistedDerivedDirectoryRecordBinding,
    ) -> Option<Self> {
        (expected_previous
            .indexed_through_blob_publication()
            .is_some()
            && next.indexed_through().is_none()
            && next.record().record() != expected_previous.directory_record())
        .then_some(Self {
            expected_previous,
            expected_previous_payload_sha256,
            next,
        })
    }

    pub const fn expected_previous(self) -> DerivedFamilyRootDirectoryBinding {
        self.expected_previous
    }

    pub const fn expected_previous_payload_sha256(self) -> [u8; 32] {
        self.expected_previous_payload_sha256
    }

    pub const fn next(self) -> PersistedDerivedDirectoryRecordBinding {
        self.next
    }
}

//! Custom rules validate logical truth, which sealed suspension does not edit.
//! The ordinary observation remains the physical candidate for native checks
//! and touched-scope capture; only custom reads use this exact before-image.

use crate::identity::data::VersionId;
use crate::storage::overlay::PartitionAccess;

use super::InvariantObservation;

impl InvariantObservation<'_> {
    pub(crate) fn custom_uses_suspension_source(&self) -> bool {
        self.proposal_identity()
            .and_then(|proposal| proposal.suspension_source_version())
            .is_some()
            && self.before_image_partition_access().is_some()
    }

    pub(crate) fn custom_enforcement_partition_access(&self) -> &dyn PartitionAccess {
        if self.custom_uses_suspension_source() {
            self.before_image_partition_access()
                .expect("sealed suspension source exists")
        } else {
            self.enforcement_partition_access()
        }
    }

    pub(crate) fn custom_committed_partition_access(&self) -> &dyn PartitionAccess {
        if self.custom_uses_suspension_source() {
            self.before_image_partition_access()
                .expect("sealed suspension source exists")
        } else {
            self.committed_partition_access()
        }
    }

    pub(crate) fn custom_enforcement_version_id(&self, fallback: VersionId) -> VersionId {
        if self.custom_uses_suspension_source() {
            self.proposal_identity()
                .and_then(|proposal| proposal.suspension_source_version())
                .expect("sealed suspension source version exists")
        } else {
            self.enforcement_version_id(fallback)
        }
    }
}

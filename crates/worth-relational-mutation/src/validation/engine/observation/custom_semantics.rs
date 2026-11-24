//! Custom rules validate logical truth, which sealed suspension does not edit.
//! The ordinary observation remains the physical candidate for native checks
//! and touched-scope capture; only custom reads use this exact before-image.

use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

use super::InvariantObservation;

impl InvariantObservation<'_> {
    fn custom_suspension_source(&self) -> Option<(&dyn CurrentPartitionAccess, VersionId)> {
        let version = self.proposal_identity()?.suspension_source_version()?;
        Some((self.before_image_partition_access()?, version))
    }

    pub fn custom_uses_suspension_source(&self) -> bool {
        self.custom_suspension_source().is_some()
    }

    pub fn custom_enforcement_partition_access(&self) -> &dyn CurrentPartitionAccess {
        self.custom_suspension_source()
            .map_or_else(|| self.enforcement_partition_access(), |(source, _)| source)
    }

    pub(crate) fn custom_committed_partition_access(&self) -> &dyn CurrentPartitionAccess {
        self.custom_suspension_source()
            .map_or_else(|| self.committed_partition_access(), |(source, _)| source)
    }

    pub fn custom_enforcement_version_id(&self, fallback: VersionId) -> VersionId {
        self.custom_suspension_source().map_or_else(
            || self.enforcement_version_id(fallback),
            |(_, version)| version,
        )
    }
}

use worth_store_physical_format::{
    BlobReclaimDescriptorV1, BlobReclaimDescriptorV2, PersistedRecordIdentity,
};

use crate::physical_runtime::durability::{
    AdmittedFailedIngestDrop, AdmittedReleasedGenerationDrop, PhysicalReclaimAttempt,
};

#[derive(Clone, Copy)]
pub(in crate::physical_runtime::blob::reclaim) enum ReclaimSource<'a> {
    Failed(&'a AdmittedFailedIngestDrop),
    Released(&'a AdmittedReleasedGenerationDrop),
}

impl ReclaimSource<'_> {
    pub(super) fn attempt(&self) -> &PhysicalReclaimAttempt {
        match self {
            Self::Failed(value) => value.attempt(),
            Self::Released(value) => value.attempt(),
        }
    }

    pub(super) fn dropped(&self) -> &[PersistedRecordIdentity] {
        match self {
            Self::Failed(value) => value.dropped(),
            Self::Released(value) => value.dropped(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ReclaimDescriptor {
    Failed(BlobReclaimDescriptorV1),
    Released(BlobReclaimDescriptorV2),
}

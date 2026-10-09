use worth_store_physical_format::{DurablePhysicalRootManifest, RootPublicationCell};

use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalEffectIdentity, RetainedPhysicalRoot,
};

pub struct PhysicalRecoveryRootBasis {
    current: DurablePhysicalRootManifest,
    previous: Option<RetainedPhysicalRoot>,
    namespace: PhysicalRootNamespaceDurabilityEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootNamespaceDurabilityEvidence {
    ReopenedCurrentRoot {
        root: RootPublicationCell,
    },
    PublishedCurrentRoot {
        group: PhysicalDurabilityGroupBasis,
        source_generation: u64,
        current_generation: u64,
        replacement: PhysicalEffectIdentity,
        namespace_synchronization: PhysicalEffectIdentity,
    },
    RetirementCurrentRoot {
        operation: crate::physical_runtime::PhysicalMutationIdentity,
        source_generation: u64,
        current_generation: u64,
        replacement: PhysicalEffectIdentity,
        namespace_synchronization: PhysicalEffectIdentity,
    },
    ManifestResidueCurrentRoot {
        operation: crate::physical_runtime::PhysicalMutationIdentity,
        manifest: worth_store_physical_format::PersistedRecordIdentity,
        source_generation: u64,
        current_generation: u64,
        replacement: PhysicalEffectIdentity,
        namespace_synchronization: PhysicalEffectIdentity,
    },
    TierEpochCurrentRoot {
        operation: crate::physical_runtime::PhysicalMutationIdentity,
        epoch: u64,
        source_generation: u64,
        current_generation: u64,
        replacement: PhysicalEffectIdentity,
        namespace_synchronization: PhysicalEffectIdentity,
    },
}

impl PhysicalRecoveryRootBasis {
    pub(in crate::physical_runtime) const fn new(
        current: DurablePhysicalRootManifest,
        previous: Option<RetainedPhysicalRoot>,
        namespace: PhysicalRootNamespaceDurabilityEvidence,
    ) -> Self {
        Self {
            current,
            previous,
            namespace,
        }
    }

    pub const fn current(&self) -> &DurablePhysicalRootManifest {
        &self.current
    }

    pub const fn previous(&self) -> Option<&RetainedPhysicalRoot> {
        self.previous.as_ref()
    }

    pub const fn namespace_evidence(&self) -> PhysicalRootNamespaceDurabilityEvidence {
        self.namespace
    }
}

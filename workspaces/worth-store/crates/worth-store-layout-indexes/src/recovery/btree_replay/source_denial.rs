use worth_store_physical_format::{
    InMemoryPhysicalFormatModelDenial, PhysicalBootstrapCatalogDenial, PhysicalReference,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BTreeReplaySourceDenial {
    BootstrapCatalog(PhysicalBootstrapCatalogDenial),
    PhysicalOpen(Box<InMemoryPhysicalFormatModelDenial>),
    RootManifestMissing,
    AmbiguousRootManifest {
        candidates: usize,
    },
    NoAdmittedDurableSource,
    DurableSourceBlockedByIntegrity,
    WalOnlyRootNotMaterialized,
    CheckpointRootMismatch {
        expected: PhysicalReference,
        actual: PhysicalReference,
    },
    CheckpointTailIdentityMismatch,
    CheckpointTailFrontierMismatch {
        checkpoint_end: u64,
        tail_start: u64,
    },
}

impl From<PhysicalBootstrapCatalogDenial> for BTreeReplaySourceDenial {
    fn from(value: PhysicalBootstrapCatalogDenial) -> Self {
        Self::BootstrapCatalog(value)
    }
}

impl From<InMemoryPhysicalFormatModelDenial> for BTreeReplaySourceDenial {
    fn from(value: InMemoryPhysicalFormatModelDenial) -> Self {
        Self::PhysicalOpen(Box::new(value))
    }
}

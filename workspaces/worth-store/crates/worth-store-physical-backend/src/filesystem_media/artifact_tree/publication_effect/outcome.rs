use super::super::{ArtifactTreeDirectory, ArtifactTreeFailure, ArtifactTreeFile};
use crate::{
    filesystem_media::{MediaOperationIdentity, MediaOwnerIdentity},
    BackendQueueExecutionCompletion,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactTreePublicationEffect {
    FileSynchronization(ArtifactTreeFile),
    DirectorySynchronization(ArtifactTreeDirectory),
    DurableRemoval(ArtifactTreeFile),
    Replacement {
        source: ArtifactTreeFile,
        destination: ArtifactTreeFile,
    },
    RootProtocolReplacement {
        previous_selector: ArtifactTreeReplacement,
        current_selector: ArtifactTreeReplacement,
        bootstrap_catalog: ArtifactTreeReplacement,
    },
}

impl ArtifactTreePublicationEffect {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        match self {
            Self::FileSynchronization(file) | Self::DurableRemoval(file) => file.owned_heap_bytes(),
            Self::DirectorySynchronization(directory) => directory.owned_heap_bytes(),
            Self::Replacement {
                source,
                destination,
            } => source
                .owned_heap_bytes()?
                .checked_add(destination.owned_heap_bytes()?),
            Self::RootProtocolReplacement {
                previous_selector,
                current_selector,
                bootstrap_catalog,
            } => previous_selector
                .owned_heap_bytes()?
                .checked_add(current_selector.owned_heap_bytes()?)?
                .checked_add(bootstrap_catalog.owned_heap_bytes()?),
        }
    }

    pub const fn is_file_synchronization(&self) -> bool {
        matches!(self, Self::FileSynchronization(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTreeReplacement {
    pub(super) source: ArtifactTreeFile,
    pub(super) destination: ArtifactTreeFile,
}

impl ArtifactTreeReplacement {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.source
            .owned_heap_bytes()?
            .checked_add(self.destination.owned_heap_bytes()?)
    }

    pub fn new(source: ArtifactTreeFile, destination: ArtifactTreeFile) -> Self {
        Self {
            source,
            destination,
        }
    }

    pub const fn source(&self) -> &ArtifactTreeFile {
        &self.source
    }

    pub const fn destination(&self) -> &ArtifactTreeFile {
        &self.destination
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedArtifactTreePublicationEffect {
    pub(super) owner: MediaOwnerIdentity,
    pub(super) store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    pub(super) operation: MediaOperationIdentity,
    pub(super) effect: ArtifactTreePublicationEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminateArtifactTreePublicationEffect {
    pub(super) failure: ArtifactTreeFailure,
    pub(super) owner: MediaOwnerIdentity,
    pub(super) store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    pub(super) operation: MediaOperationIdentity,
    pub(super) effect: ArtifactTreePublicationEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactTreePublicationEffectOutcome {
    Completed(CompletedArtifactTreePublicationEffect),
    DeniedBeforeEffect(ArtifactTreeFailure),
    Indeterminate(IndeterminateArtifactTreePublicationEffect),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedScheduledArtifactTreePublicationEffect {
    pub(super) physical: CompletedArtifactTreePublicationEffect,
    pub(super) queue: BackendQueueExecutionCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduledArtifactTreePublicationEffectOutcome {
    Completed(Box<CompletedScheduledArtifactTreePublicationEffect>),
    DeniedBeforeEffect(ArtifactTreeFailure),
    Indeterminate(IndeterminateArtifactTreePublicationEffect),
}

impl CompletedArtifactTreePublicationEffect {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.effect.owned_heap_bytes()
    }

    pub const fn owner(&self) -> MediaOwnerIdentity {
        self.owner
    }

    pub const fn store(&self) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.store
    }

    pub const fn operation(&self) -> MediaOperationIdentity {
        self.operation
    }

    pub const fn effect(&self) -> &ArtifactTreePublicationEffect {
        &self.effect
    }
}

impl IndeterminateArtifactTreePublicationEffect {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.effect.owned_heap_bytes()
    }

    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.failure
    }

    pub const fn owner(&self) -> MediaOwnerIdentity {
        self.owner
    }

    pub const fn store(&self) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.store
    }

    pub const fn operation(&self) -> MediaOperationIdentity {
        self.operation
    }

    pub const fn effect(&self) -> &ArtifactTreePublicationEffect {
        &self.effect
    }
}

impl CompletedScheduledArtifactTreePublicationEffect {
    pub const fn physical(&self) -> &CompletedArtifactTreePublicationEffect {
        &self.physical
    }

    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
}

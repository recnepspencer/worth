use serde::{Deserialize, Serialize};
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily as Family;
use worth_store_physical_integrity::PhysicalArtifactScope;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessScopeIdentity {
    Bootstrap,
    CurrentSelector,
    PreviousSelector,
    Root {
        generation: u64,
    },
    RootBlock {
        tree: u64,
        generation: u64,
        block: u64,
    },
    SegmentBlock {
        tree: u64,
        generation: u64,
        block: u64,
    },
    FreeSpace {
        tree: u64,
        generation: u64,
    },
    FreeSpaceBlock {
        tree: u64,
        generation: u64,
        block: u64,
    },
    Page {
        segment: u64,
        page: u64,
        generation: u64,
    },
    Extent {
        extent: u64,
        generation: u64,
    },
    Chunk {
        extent: u64,
        generation: u64,
        ordinal: u32,
    },
    Wal {
        segment: u64,
        generation: u64,
    },
    Checkpoint {
        sequence: Option<u64>,
    },
    PhysicalWork {
        runtime: u64,
        generation: u64,
        operation: u64,
    },
}

pub(super) fn project(scope: PhysicalArtifactScope) -> ProcessScopeIdentity {
    match scope.artifact_family() {
        Family::NamespaceIdentity => unreachable!("C.4 owns namespace admission"),
        Family::BootstrapCatalog => ProcessScopeIdentity::Bootstrap,
        Family::CurrentRootSelector => ProcessScopeIdentity::CurrentSelector,
        Family::PreviousRootSelector => ProcessScopeIdentity::PreviousSelector,
        Family::RootManifest => ProcessScopeIdentity::Root {
            generation: scope.root_generation().unwrap(),
        },
        Family::RootRoutingBlock => {
            let identity = scope.root_routing_block_identity().unwrap();
            ProcessScopeIdentity::RootBlock {
                tree: identity.tree().get(),
                generation: identity.reference().generation(),
                block: identity.reference().block(),
            }
        }
        Family::SegmentMembership => {
            let identity = scope.segment_membership_block_identity().unwrap();
            ProcessScopeIdentity::SegmentBlock {
                tree: identity.tree().get(),
                generation: identity.reference().generation(),
                block: identity.reference().block(),
            }
        }
        Family::FreeSpaceHeader => {
            let identity = scope.free_space_header_identity().unwrap();
            ProcessScopeIdentity::FreeSpace {
                tree: identity.tree().get(),
                generation: identity.generation().get(),
            }
        }
        Family::FreeSpaceMembershipBlock => {
            let identity = scope.free_space_membership_block_identity().unwrap();
            ProcessScopeIdentity::FreeSpaceBlock {
                tree: identity.tree().get(),
                generation: identity.reference().generation(),
                block: identity.reference().block(),
            }
        }
        Family::PageFrame => {
            let identity = scope.page_identity().unwrap();
            ProcessScopeIdentity::Page {
                segment: identity.segment_id().get(),
                page: identity.page_id().get(),
                generation: identity.generation().get(),
            }
        }
        Family::ExtentManifest => {
            let identity = scope.extent_manifest_placement().unwrap();
            ProcessScopeIdentity::Extent {
                extent: identity.extent().get(),
                generation: identity.extent_generation(),
            }
        }
        Family::ExtentChunk => {
            let identity = scope.extent_chunk_coordinate().unwrap();
            ProcessScopeIdentity::Chunk {
                extent: identity.extent_cell().extent_id().get(),
                generation: identity.extent_cell().generation().get(),
                ordinal: identity.ordinal(),
            }
        }
        Family::WalFrame => {
            let identity = scope.wal_segment_identity().unwrap();
            ProcessScopeIdentity::Wal {
                segment: identity.segment().get(),
                generation: identity.generation().get(),
            }
        }
        Family::CheckpointStreamHeader
        | Family::CheckpointDirtyBasis
        | Family::CheckpointBindingCompaction
        | Family::CheckpointBinding
        | Family::CheckpointFooter => ProcessScopeIdentity::Checkpoint {
            sequence: scope
                .checkpoint_identity()
                .map(|identity| identity.sequence().get()),
        },
        Family::PhysicalWorkObligation => {
            let identity = scope.physical_work_obligation_identity().unwrap();
            ProcessScopeIdentity::PhysicalWork {
                runtime: identity.runtime().get(),
                generation: identity.generation().get(),
                operation: identity.operation().get(),
            }
        }
    }
}

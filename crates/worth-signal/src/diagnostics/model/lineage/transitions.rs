mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::handle::NodeId;
use crate::data::output::OutputChange;
use crate::data::reuse::PersistentCorrespondenceKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactTransitionKind {
    Replaced,
    Refreshed {
        output_change: OutputChange,
    },
    MemoizedReuse,
    SnapshotRestoreReuse,
    ReconciliationAdoption,
    CrossIdentityPersistentReuse {
        correspondence_kind: PersistentCorrespondenceKind,
    },
    PartialArtifactSplice {
        composition_region_count: u32,
        recomputed_region_count: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotRestoreKind {
    CompactGlobal,
    PerNodeArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidationCause {
    SourceAspectChanged {
        aspect_index: usize,
    },
    DirectDependencyChanged {
        dependency: NodeId,
        aspect_index: usize,
    },
    TransitiveDependencyChanged {
        aspect_index: usize,
    },
    PendingDependencyRevalidation {
        upstream: Option<NodeId>,
    },
}

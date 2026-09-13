mod candidate;
mod checkpoint_base;
mod checkpoint_covered_wal;
mod compaction_product;
mod current_previous_root;
mod page_facts;
mod page_lsn_skip_apply;
mod physical_source;
mod residue;
mod selection;
mod structured_observation;
mod wal_segment_disposition;
mod wal_tail;

pub use candidate::{
    PhysicalRootCandidateDenial, PhysicalRootManifestDenial, PhysicalRootSelectorDenial,
    PhysicalRootSlotObservation, PhysicalRootSourceCandidate,
};
pub use checkpoint_base::{PhysicalCheckpointBase, PhysicalCheckpointBaseDenial};
pub use checkpoint_covered_wal::CheckpointCoveredWalArtifact;
pub use compaction_product::SelectedCompactionProduct;
pub use current_previous_root::{
    select_current_previous_root, PhysicalBootstrapFallbackAnchor, PhysicalRootSelectionDenial,
    SelectedPhysicalRoot, SelectedPhysicalRootRole,
};
pub use page_facts::{
    admit_physical_page_facts, PhysicalManifestBlockProjection, PhysicalPageFactDenial,
    SelectedPhysicalPageFacts,
};
pub use page_lsn_skip_apply::PageLsnSkipApplyDecision;
pub use physical_source::PhysicalRecoverySource;
pub use residue::{PhysicalRecoveryResidue, PhysicalRecoveryResidueKind};
pub use selection::{
    select_physical_recovery_sources, PhysicalSourceSelection, PhysicalSourceSelectionDenial,
    PhysicalSourceSelectionTrace,
};
pub use structured_observation::observe_structured_physical_root_candidate;
pub use wal_segment_disposition::{
    classify_admitted_wal_segment, AdmittedWalFrameRejectionKind, AdmittedWalSegmentPolicyInput,
    PhysicalWalSegmentDisposition,
};
pub use wal_tail::{
    admit_physical_wal_tail, PhysicalWalFrameFacts, PhysicalWalInterruptionFacts,
    PhysicalWalSegmentCandidate, SelectedPhysicalWalTail, SelectedPhysicalWalTailDenial,
};

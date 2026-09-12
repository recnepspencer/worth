mod evidence;
mod freshness;
mod kinds;
mod ordering;
mod publication;
mod scopes;
mod vector;

pub use evidence::{
    compare_physical_epoch_vectors_with_evidence, PhysicalEpochComparisonEvidence,
    PhysicalEpochComparisonEvidenceDenial, PhysicalEpochFreshnessBasis,
    PhysicalEpochFreshnessProofArtifact, PhysicalEpochFreshnessProofEvidence,
    PhysicalEpochFreshnessProofPhase,
};
pub use freshness::{
    EpochRetryDecision, PhysicalEpochDriftKind, PhysicalEpochFreshness, StalePhysicalReadPlanDenial,
};
pub(crate) use kinds::{
    chunk_epoch_from_future_publication, extent_epoch_from_publication,
    page_epoch_from_publication, segment_epoch_from_publication,
};
#[cfg(any(test, feature = "certification-authority"))]
pub(crate) use kinds::{manifest_epoch_from_entry_seed, root_epoch_from_entry_seed};
pub use kinds::{ChunkEpoch, ExtentEpoch, ManifestEpoch, PageEpoch, RootEpoch, SegmentEpoch};

#[cfg(any(test, feature = "certification-authority"))]
pub fn next_root_epoch_for_certification(source: RootEpoch) -> RootEpoch {
    RootEpoch::from_admitted_physical_basis(
        source
            .get()
            .checked_add(1)
            .expect("certification root epoch must have a successor"),
    )
}
pub use ordering::{
    required_physical_isolation_ordering_contracts, PhysicalOrderingContract,
    PhysicalOrderingContractDenial, PhysicalOrderingSite, PhysicalOrderingStrength,
};
pub use publication::{
    ExtentPublicationEpochBasis, FutureChunkPublicationEpochBasis, PagePublicationEpochBasis,
    SegmentPublicationEpochBasis,
};
pub use scopes::{EpochComparisonScope, EpochComparisonScopeMismatch, EpochStabilityScopeKind};
pub use vector::{PhysicalEpochVector, PhysicalEpochVectorBuilder, PhysicalEpochVectorDenial};

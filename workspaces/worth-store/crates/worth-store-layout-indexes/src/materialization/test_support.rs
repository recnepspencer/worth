use worth_store_physical_format::CheckpointWalSourceRange;
use worth_store_physical_format::PhysicalEpoch;
use worth_store_wal::LogSequenceNumber;

use super::{
    CoverageGapWitness, LayoutCoverageWitness, LayoutMaterializationSourceIdentity,
    LayoutMaterializationState, MaterializationDenial, PhysicalCoverageBasis,
};
use crate::blob_basis::BlobGenerationBasis;
use crate::catalog::PhysicalArtifactFamilyDeclaration;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MaterializationObservationFixtures;

pub(crate) const fn materialization_observations() -> MaterializationObservationFixtures {
    MaterializationObservationFixtures
}

impl MaterializationObservationFixtures {
    fn catalog_source(self) -> LayoutMaterializationSourceIdentity {
        let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
        LayoutMaterializationSourceIdentity::from_catalog(&catalog)
    }

    pub(crate) fn exact_root_epoch_coverage(
        self,
        state: LayoutMaterializationState,
        epoch: PhysicalEpoch,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        LayoutCoverageWitness::observed_exact_through(
            state,
            PhysicalCoverageBasis::root_epoch(epoch).watermark(),
            self.catalog_source(),
        )
    }

    pub(crate) fn exact_wal_lsn_coverage(
        self,
        state: LayoutMaterializationState,
        lsn: LogSequenceNumber,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        LayoutCoverageWitness::observed_exact_through(
            state,
            PhysicalCoverageBasis::wal_lsn(lsn).watermark(),
            self.catalog_source(),
        )
    }

    pub(crate) fn exact_blob_generation_coverage(
        self,
        state: LayoutMaterializationState,
        generation: BlobGenerationBasis,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        LayoutCoverageWitness::observed_exact_through(
            state,
            PhysicalCoverageBasis::blob_generation(generation).watermark(),
            self.catalog_source(),
        )
    }

    pub(crate) fn exact_checkpoint_coverage(
        self,
        state: LayoutMaterializationState,
        range: CheckpointWalSourceRange,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        LayoutCoverageWitness::observed_exact_through(
            state,
            PhysicalCoverageBasis::checkpoint_frontier(range).watermark(),
            self.catalog_source(),
        )
    }

    pub(crate) fn stale_root_epoch_coverage(
        self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        epoch: PhysicalEpoch,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        self.exact_root_epoch_coverage(
            LayoutMaterializationState::stale(declaration.family()),
            epoch,
        )
    }

    pub(crate) fn lagged_wal_lsn_coverage(
        self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        lower_bound: LogSequenceNumber,
        upper_bound: LogSequenceNumber,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        LayoutCoverageWitness::observed_lagged(
            LayoutMaterializationState::lagged(declaration.family()),
            PhysicalCoverageBasis::wal_lsn(lower_bound).watermark(),
            PhysicalCoverageBasis::wal_lsn(upper_bound).watermark(),
            self.catalog_source(),
        )
    }

    pub(crate) fn partial_wal_lsn_coverage(
        self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        lower_bound: LogSequenceNumber,
        upper_bound: LogSequenceNumber,
        gap: CheckpointWalSourceRange,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        self.gapped_wal_lsn_coverage(declaration, lower_bound, upper_bound, gap, false)
    }

    pub(crate) fn quarantined_wal_lsn_coverage(
        self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        lower_bound: LogSequenceNumber,
        upper_bound: LogSequenceNumber,
        gap: CheckpointWalSourceRange,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        self.gapped_wal_lsn_coverage(declaration, lower_bound, upper_bound, gap, true)
    }

    fn gapped_wal_lsn_coverage(
        self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        lower_bound: LogSequenceNumber,
        upper_bound: LogSequenceNumber,
        gap: CheckpointWalSourceRange,
        quarantined: bool,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        let witness = CoverageGapWitness::physical_range(
            declaration.family(),
            PhysicalCoverageBasis::checkpoint_frontier(gap).basis_kind(),
            gap.admitted_begin_lsn(),
            gap.covered_end_lsn_exclusive(),
        );
        let state = if quarantined {
            LayoutMaterializationState::quarantined(declaration.family())
        } else {
            LayoutMaterializationState::partially_covered(declaration.family())
        };
        LayoutCoverageWitness::observed_partial(
            state,
            PhysicalCoverageBasis::wal_lsn(lower_bound).watermark(),
            PhysicalCoverageBasis::wal_lsn(upper_bound).watermark(),
            witness,
            self.catalog_source(),
        )
    }
}

impl crate::planning::AccessPlanningFacade {
    pub(crate) fn exact_root_epoch_coverage(
        &self,
        state: LayoutMaterializationState,
        epoch: PhysicalEpoch,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        materialization_observations().exact_root_epoch_coverage(state, epoch)
    }

    pub(crate) fn exact_wal_lsn_coverage(
        &self,
        state: LayoutMaterializationState,
        lsn: LogSequenceNumber,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        materialization_observations().exact_wal_lsn_coverage(state, lsn)
    }

    pub(crate) fn exact_blob_generation_coverage(
        &self,
        state: LayoutMaterializationState,
        generation: BlobGenerationBasis,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        materialization_observations().exact_blob_generation_coverage(state, generation)
    }

    pub(crate) fn exact_checkpoint_coverage(
        &self,
        state: LayoutMaterializationState,
        range: CheckpointWalSourceRange,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        materialization_observations().exact_checkpoint_coverage(state, range)
    }

    pub(crate) fn stale_root_epoch_coverage(
        &self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        epoch: PhysicalEpoch,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        materialization_observations().stale_root_epoch_coverage(declaration, epoch)
    }

    pub(crate) fn lagged_wal_lsn_coverage(
        &self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        lower_bound: LogSequenceNumber,
        upper_bound: LogSequenceNumber,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        materialization_observations().lagged_wal_lsn_coverage(
            declaration,
            lower_bound,
            upper_bound,
        )
    }

    pub(crate) fn partial_wal_lsn_coverage(
        &self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        lower_bound: LogSequenceNumber,
        upper_bound: LogSequenceNumber,
        gap: CheckpointWalSourceRange,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        materialization_observations().partial_wal_lsn_coverage(
            declaration,
            lower_bound,
            upper_bound,
            gap,
        )
    }

    pub(crate) fn quarantined_wal_lsn_coverage(
        &self,
        declaration: &'static PhysicalArtifactFamilyDeclaration,
        lower_bound: LogSequenceNumber,
        upper_bound: LogSequenceNumber,
        gap: CheckpointWalSourceRange,
    ) -> Result<LayoutCoverageWitness, MaterializationDenial> {
        materialization_observations().quarantined_wal_lsn_coverage(
            declaration,
            lower_bound,
            upper_bound,
            gap,
        )
    }
}

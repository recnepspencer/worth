use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{
    WorthQueryGraphCompositionAssumptionSummary, WorthQueryGraphCompositionEvidence,
    WorthQueryGraphCompositionLifecycleOutcomes, WorthQueryGraphCompositionLineageSummary,
    WorthQueryGraphCompositionResolutionMap,
};

use super::WorthQueryBatchWriteReceipt;

impl WorthQueryBatchWriteReceipt {
    pub fn graph_composition_evidence(
        &self,
    ) -> Result<Option<WorthQueryGraphCompositionEvidence>, IdentityDerivationDenial> {
        let lifecycle_outcomes =
            WorthQueryGraphCompositionLifecycleOutcomes::derive(&self.graph_composition_program)?;
        Ok(WorthQueryGraphCompositionEvidence::derive(
            &self.write_receipts,
            &self.graph_composition_breadth,
            &lifecycle_outcomes,
            &self.graph_composition_resolution_map,
            self.affected_live_view_targets.len(),
            self.affected_derived_view_targets.len(),
            self.considered_computed_view_count,
        )?)
    }

    pub fn graph_composition_assumption_summary(
        &self,
    ) -> Result<Option<WorthQueryGraphCompositionAssumptionSummary>, IdentityDerivationDenial> {
        if self.graph_composition_program.is_empty() {
            return Ok(None);
        }
        Ok(WorthQueryGraphCompositionAssumptionSummary::derive(
            &self.write_receipts,
        )?)
    }

    pub fn graph_composition_lineage_summary(
        &self,
    ) -> Result<Option<WorthQueryGraphCompositionLineageSummary>, IdentityDerivationDenial> {
        if self.graph_composition_program.is_empty() {
            return Ok(None);
        }
        Ok(WorthQueryGraphCompositionLineageSummary::derive(
            &self.write_receipts,
        )?)
    }

    pub fn graph_composition_lifecycle_outcomes(
        &self,
    ) -> Option<WorthQueryGraphCompositionLifecycleOutcomes> {
        WorthQueryGraphCompositionLifecycleOutcomes::derive(&self.graph_composition_program)
    }

    pub fn graph_composition_resolution_map(&self) -> &WorthQueryGraphCompositionResolutionMap {
        &self.graph_composition_resolution_map
    }
}

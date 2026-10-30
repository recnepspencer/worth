use super::assembly::SelectedEvidence;
use super::assembly_support::{
    child_intrinsic_host_compatibility, child_intrinsic_query_compatibility,
    host_result_compatibility, settled_query_receipt_compatibility,
};
use crate::evidence::UiMeasurementGenerationCompatibility;
use crate::graph::UiGraphWorldProfile;
use worth_ui_inspection::UiEvidenceAuthorityGeneration;

impl SelectedEvidence<'_> {
    pub(super) fn generation_compatibility(
        &self,
        world_profile: &UiGraphWorldProfile,
        declaration_support_authority_generation: UiEvidenceAuthorityGeneration,
    ) -> UiMeasurementGenerationCompatibility {
        if let Some(receipt) = self.query_receipt {
            let compatibility = settled_query_receipt_compatibility(
                receipt,
                world_profile,
                declaration_support_authority_generation,
            );
            if let Some(compatibility) = compatibility {
                return compatibility;
            }
        }
        if let Some(report) = self.host_capability_report {
            for result in self.host_results.relevant_results().into_iter().flatten() {
                if let Some(compatibility) = host_result_compatibility(
                    Some(result),
                    report,
                    declaration_support_authority_generation,
                ) {
                    return compatibility;
                }
            }
        }
        for evidence in &self.child_intrinsic_measurements {
            if let Some(compatibility) = child_intrinsic_query_compatibility(
                evidence,
                world_profile,
                declaration_support_authority_generation,
            ) {
                return compatibility;
            }
            if let Some(report) = self.host_capability_report
                && let Some(compatibility) = child_intrinsic_host_compatibility(
                    evidence,
                    report,
                    declaration_support_authority_generation,
                )
            {
                return compatibility;
            }
        }
        UiMeasurementGenerationCompatibility::Compatible
    }
}

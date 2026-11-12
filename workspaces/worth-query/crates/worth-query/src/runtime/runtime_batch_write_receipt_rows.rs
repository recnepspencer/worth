use super::runtime_batching::BatchCommandSummary;
use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl WorthQueryRuntime {
    pub(super) fn build_batch_component_write_receipts(
        &self,
        receipts: Vec<WorthQueryMutationReceipt>,
        command_summaries: Vec<BatchCommandSummary>,
    ) -> Result<Vec<WorthQueryWriteReceipt>, IdentityDerivationDenial> {
        receipts
            .into_iter()
            .zip(command_summaries)
            .map(|(receipt, summary)| {
                let mutation_family = summary.mutation_family();
                let declared_collection_identity = summary.declared_collection_identity();
                let declared_entity_identity = summary.declared_entity_identity();
                let existing_truth_binding = summary.existing_truth_binding();
                let verified_existing_truth_assertion = summary.verified_existing_truth_assertion();
                let symbolic_target_reference = summary.symbolic_target_reference();
                let naming_intent = summary.naming_intent();
                let continuity_intent = summary.continuity_intent();
                let declared_aspect_operations = summary.declared_aspect_operations();
                let declared_aspect_value_digest = summary.declared_aspect_value_digest();
                let symbolic_aspect_resolution_evidence =
                    summary.symbolic_aspect_resolution_evidence();
                let mutation_metadata = summary.mutation_metadata();
                let affected_live_view_targets = self.backend.affected_live_view_targets(&receipt);
                let (_, target_collection, mut target_entity_identity) =
                    classify_receipt_mutation_summary(&receipt);
                let mut target_collection_identity = target_collection;
                if let Some(binding) = existing_truth_binding.as_ref() {
                    target_collection_identity = binding.target_collection_identity().cloned();
                    target_entity_identity = Some(binding.resolved_entity_artifact_identity());
                }
                WorthQueryWriteReceipt::batch_component(
                    receipt,
                    mutation_family,
                    WorthQueryAuthorityLane::AuthoritativeTruth,
                    declared_collection_identity,
                    declared_entity_identity,
                    existing_truth_binding,
                    verified_existing_truth_assertion,
                    symbolic_target_reference,
                    symbolic_aspect_resolution_evidence,
                    naming_intent,
                    continuity_intent,
                    target_collection_identity,
                    target_entity_identity,
                    declared_aspect_operations,
                    declared_aspect_value_digest,
                    mutation_metadata,
                    affected_live_view_targets,
                    WorthQueryAuthorityLane::AuthoritativeTruth,
                )
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;

use crate::merge::data::{
    MergeArtifactDigestBasis, MergePlanningDecisionLogDigestBasis, PreparedMergeExecution,
    RelationalMergeAdmittedSurfaceRow, RelationalMergeCorrespondenceWitness,
    RelationalMergeProofPacket, RelationalMergeStrategyWitness,
    RelationalSchemaReconciliationWitness,
};

use super::MergeAccess;

impl<'runtime> MergeAccess<'runtime> {
    pub fn retain_merge_proof_packet_from_prepared_execution(
        &self,
        prepared: &PreparedMergeExecution,
    ) -> RelationalMergeProofPacket {
        let correspondence_witness =
            self.retain_merge_correspondence_witness_from_prepared_execution(prepared);
        let schema_reconciliation_witness =
            self.retain_schema_reconciliation_witness_from_prepared_execution(prepared);
        let strategy_witness = self.retain_merge_strategy_witness_from_prepared_execution(prepared);
        self.retain_merge_proof_packet_from_prepared_execution_with_witness(
            prepared,
            &correspondence_witness,
            &schema_reconciliation_witness,
            &strategy_witness,
        )
    }

    pub(crate) fn retain_merge_proof_packet_from_prepared_execution_with_witness(
        &self,
        prepared: &PreparedMergeExecution,
        correspondence_witness: &RelationalMergeCorrespondenceWitness,
        schema_reconciliation_witness: &RelationalSchemaReconciliationWitness,
        strategy_witness: &RelationalMergeStrategyWitness,
    ) -> RelationalMergeProofPacket {
        self.retain_merge_proof_packet_from_compiled_execution_with_witness(
            prepared.compiled(),
            correspondence_witness,
            schema_reconciliation_witness,
            strategy_witness,
        )
    }

    pub(crate) fn retain_merge_proof_packet_from_compiled_execution_with_witness(
        &self,
        compiled: &crate::merge::data::CompiledMergeExecution,
        correspondence_witness: &RelationalMergeCorrespondenceWitness,
        schema_reconciliation_witness: &RelationalSchemaReconciliationWitness,
        strategy_witness: &RelationalMergeStrategyWitness,
    ) -> RelationalMergeProofPacket {
        let foundational_request =
            self.lower_merge_request_to_foundational(compiled.request().clone());
        let TransitionOutcome::Success(foundational_request) = foundational_request else {
            panic!(
                /* Proof: CompiledMergeExecution can be constructed only after this same request lowers successfully to foundational vocabulary; its request is immutable thereafter. */
                "prepared execution request must lower to foundational merge vocabulary"
            );
        };

        RelationalMergeProofPacket::retained_execution_admitted(
            compiled.request().clone(),
            compiled.execution_ready_plan().basis.clone(),
            admitted_merge_surface(compiled.execution_ready_plan()),
            correspondence_witness.witness_digest().to_string(),
            schema_reconciliation_witness.witness_digest().to_string(),
            strategy_witness.witness_digest().to_string(),
            foundational_request.lowering_digest().to_string(),
            planning_digest(
                &compiled.artifact().digest_basis,
                &compiled.artifact().decision_log_digest_basis,
            ),
            compiled
                .bound_executable_plan()
                .authority_binding
                .executable_plan_digest
                .clone(),
        )
    }
}

fn admitted_merge_surface(
    execution_ready_plan: &crate::merge::data::ExecutionReadyLoweredMergePlan,
) -> Arc<[RelationalMergeAdmittedSurfaceRow]> {
    execution_ready_plan
        .lowered_records
        .iter()
        .map(|record| {
            RelationalMergeAdmittedSurfaceRow::new(
                record.record.clone(),
                record.target_record.clone(),
            )
        })
        .collect::<Vec<_>>()
        .into()
}

fn planning_digest(
    digest_basis: &MergeArtifactDigestBasis,
    decision_log_digest_basis: &MergePlanningDecisionLogDigestBasis,
) -> String {
    let bytes = rmp_serde::to_vec_named(&(digest_basis, decision_log_digest_basis))
        .expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "merge planning digest basis must encode");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;

use crate::merge::data::{
    MergeArtifactDigestBasis, MergePlanningDecisionLogDigestBasis,
    RelationalMergeAdmittedSurfaceRow, RelationalMergeCorrespondenceWitness,
    RelationalMergeProofPacket, RelationalMergeStrategyWitness,
    RelationalSchemaReconciliationWitness,
};

pub(super) fn retain_merge_proof_packet_from_compiled_execution_with_witness(
    compiled: &crate::merge::data::CompiledMergeExecution,
    correspondence_witness: &RelationalMergeCorrespondenceWitness,
    schema_reconciliation_witness: &RelationalSchemaReconciliationWitness,
    strategy_witness: &RelationalMergeStrategyWitness,
) -> RelationalMergeProofPacket {
    let foundational_request =
        super::request_foundational_lowering::lower_merge_request_to_foundational(
            compiled.request().clone(),
        );
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

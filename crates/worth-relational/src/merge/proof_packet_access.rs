use super::MergeAccess;
use crate::merge::data::{
    PreparedMergeExecution, RelationalMergeCorrespondenceWitness, RelationalMergeProofPacket,
    RelationalMergeStrategyWitness, RelationalSchemaReconciliationWitness,
};
use worth_relational_merge::internal_port::merge::data::prepared_execution::PreparedMergeExecutionOwnerPort as _;

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
        super::proof_packet::retain_merge_proof_packet_from_compiled_execution_with_witness(
            prepared.compiled(),
            correspondence_witness,
            schema_reconciliation_witness,
            strategy_witness,
        )
    }

    pub fn lower_merge_proof_packet_to_foundational_canonical_basis(
        &self,
        packet: &RelationalMergeProofPacket,
    ) -> worth_proof::TransitionOutcome<
        crate::merge::data::RelationalMergeProofPacketCanonicalBasis,
        worth_foundational::CanonicalBasisConstructionDenial,
    > {
        super::proof_packet_canonical::lower_merge_proof_packet_to_foundational_canonical_basis(
            packet,
        )
    }
}

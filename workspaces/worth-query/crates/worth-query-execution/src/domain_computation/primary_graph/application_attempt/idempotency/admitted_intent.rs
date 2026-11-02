//! One canonical admission extension for commit, resolution and process-local enrollment.
use super::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;

impl WorthQueryApplicationIdempotencyBinding {
    pub(in crate::domain_computation::primary_graph) fn bind_admission<S, O, I, E>(
        self,
        admission: &WorthQueryAdmittedApplicationOperation<S, O, I, E>,
    ) -> Self {
        self.bind_operation(admission.operation_definition_identity())
            .bind_operation_scope(admission.operation_scope_binding())
            .bind_preconditions(admission.mutation_preconditions().identity())
            .bind_governed_input(admission.governed_input_identity())
            .bind_governed_proposal(admission.governed_proposal_identity())
    }

    pub(in crate::domain_computation::primary_graph) fn canonical_key(&self) -> [u8; 32] {
        self.key_identity
    }

    /// Process-local records use the same complete intent slots as Native durable
    /// retries. Operational source-partition and producer bookkeeping is excluded
    /// by that existing encoding, rather than compared through full binding Eq.
    pub(in crate::domain_computation::primary_graph) fn has_same_intent(self, other: Self) -> bool {
        self.intent_slots() == other.intent_slots()
    }
}

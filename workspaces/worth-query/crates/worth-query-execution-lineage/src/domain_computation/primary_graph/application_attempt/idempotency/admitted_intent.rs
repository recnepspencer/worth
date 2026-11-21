//! Canonical admission extension for commit, resolution and process-local enrollment.
use super::WorthQueryApplicationIdempotencyBinding;
use crate::ports::AdmittedOperationPort;

impl WorthQueryApplicationIdempotencyBinding {
    pub fn bind_admission<A: AdmittedOperationPort + ?Sized>(self, admission: &A) -> Self {
        self.bind_operation(admission.operation_definition_identity())
            .bind_operation_scope(admission.operation_scope_binding())
            .bind_preconditions(admission.mutation_preconditions_identity())
            .bind_governed_input(admission.governed_input_identity())
            .bind_governed_proposal(admission.governed_proposal_identity())
    }

    pub const fn canonical_key(&self) -> [u8; 32] {
        self.key_identity
    }

    pub fn has_same_intent(self, other: Self) -> bool {
        self.intent_slots() == other.intent_slots()
    }
}

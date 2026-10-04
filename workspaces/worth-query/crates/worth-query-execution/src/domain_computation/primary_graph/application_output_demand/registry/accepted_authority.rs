//! Exact accepted predecessors for dependency-driven recomputation.
use super::WorthQueryAcceptedOutputAuthority;

impl WorthQueryAcceptedOutputAuthority {
    pub(in crate::domain_computation::primary_graph) fn is_same_output(
        &self,
        other: &Self,
    ) -> bool {
        match (self, other) {
            (Self::Committed(left), Self::Committed(right)) => {
                left.is_same_authoritative_commit(right)
            }
            (Self::Restored(left), Self::Restored(right)) => {
                let a = &left.checkpoint;
                let b = &right.checkpoint;
                // Payload equality is not identity and may scan large fact transports.
                a.producer == b.producer
                    && a.source == b.source
                    && a.scope == b.scope
                    && a.source_partition == b.source_partition
                    && a.producer_dependency == b.producer_dependency
                    && a.idempotency_key == b.idempotency_key
                    && left.source_scope == right.source_scope
                    && left.source_identity == right.source_identity
                    && left.observation == right.observation
            }
            _ => false,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn idempotency_key(&self) -> [u8; 32] {
        match self {
            Self::Committed(receipt) => *receipt.idempotency_binding().key_identity(),
            Self::Restored(restored) => restored.checkpoint.idempotency_key,
        }
    }
}

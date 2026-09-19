use crate::identity::data::EntityId;
use crate::symbols::data::ClientKeySymbolPolicy;
use crate::transactions::data::RecordRef;
use crate::transactions::data::TransactionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationStateInconsistencyEvidence {
    EntityCascadeDelete {
        entity_id: EntityId,
        missing: EntityCascadeDeleteMissingState,
    },
    BulkMutationAdmission {
        transaction_id: TransactionId,
        denial: BulkMutationAdmissionDenial,
    },
    MaterializationTransition {
        record: RecordRef,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityCascadeDeleteMissingState {
    Slot,
    KindId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkMutationAdmissionDenial {
    NamingPlanMismatch {
        expected_count: usize,
        actual_count: usize,
    },
    NamingPolicyViolation {
        client_key_symbol_policy: ClientKeySymbolPolicy,
    },
    NamingDigestMismatch {
        expected_digest: String,
        actual_digest: String,
    },
    LineagePlanMismatch {
        expected_count: usize,
        actual_count: usize,
    },
    LineageDigestMismatch {
        expected_digest: String,
        actual_digest: String,
    },
    TopologyRewriteRequiresLineage,
    ProvenancePlanMismatch {
        expected_batch_count: usize,
        actual_batch_count: usize,
    },
    ProvenanceDigestMismatch {
        expected_digest: String,
        actual_digest: String,
    },
}

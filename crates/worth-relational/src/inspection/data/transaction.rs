use serde::{Deserialize, Serialize};

use crate::history::data::BranchId;
use crate::transactions::data::{RecordRef, SavepointId, TransactionId};

use super::{InspectionAccessPath, InspectionAvailability, InspectionOrigin};

/// How many staged intents of each kind one transaction is carrying.
///
/// A revalidation demand is counted apart from the mutations because it is not
/// one: it brings an unchanged record back under judgement and authors nothing.
/// Folding it into `entity_mutation_count` would tell a caller reading this
/// surface that records are about to change when none are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransactionIntentCounts {
    pub create_count: u64,
    pub entity_mutation_count: u64,
    pub entity_revalidation_count: u64,
    pub relation_mutation_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavepointInspectionSurface {
    pub savepoint_id: SavepointId,
    pub retained_batch_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct TransactionInspectionSurface {
    pub transaction_id: TransactionId,
    pub target_branch: Option<BranchId>,
    pub batch_count: u64,
    pub savepoints: Vec<SavepointInspectionSurface>,
    pub touched_records: Vec<RecordRef>,
    pub intent_counts: TransactionIntentCounts,
    pub reserved_bulk_entity_slots: u64,
    pub reserved_bulk_relation_slots: u64,
    pub contains_lineage_affecting_intents: bool,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub availability: InspectionAvailability,
}

//! The original symbol and record batches used by one mutation.
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings;

#[derive(Debug)]
pub struct PendingMutationIdentities {
    pub records: PendingRecordAllocations,
    pub symbols: PendingSymbolMeanings,
}

//! The original symbol and record batches used by one mutation.
use crate::runtime::state::subsystems::record_identity::PendingRecordAllocations;
use crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings;

#[derive(Debug)]
pub(crate) struct PendingMutationIdentities {
    pub(crate) records: PendingRecordAllocations,
    pub(crate) symbols: PendingSymbolMeanings,
}

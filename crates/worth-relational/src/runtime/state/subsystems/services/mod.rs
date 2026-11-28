pub(crate) use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation;
pub(crate) mod sequence;
pub(crate) use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table;
#[cfg(test)]
mod tests;

use crate::runtime::state::subsystems::RuntimeSubsystem;

pub(crate) use instrumentation::RuntimeInstrumentation;
use sequence::RuntimeSequenceState;
#[cfg(test)]
pub(crate) use symbol_table::SymbolReservationDenial;
pub(crate) use symbol_table::{PendingSymbolMeanings, RuntimeSymbolTable};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeServices {
    sequence: RuntimeSequenceState,
    pub(crate) instrumentation: RuntimeInstrumentation,
    pub(crate) symbols: RuntimeSymbolTable,
}

impl RuntimeServices {
    pub(crate) fn preparation_binding(&self) -> Self {
        self.clone()
    }

    pub(crate) fn detached_for_preflight(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        Ok(Self {
            sequence: self.sequence.detached_preserving_runtime_id(),
            instrumentation: self.instrumentation.fork(),
            symbols: self.symbols.detached_owner_snapshot_for_request(request)?,
        })
    }
    fn empty() -> Self {
        Self {
            sequence: RuntimeSequenceState::new(),
            instrumentation: RuntimeInstrumentation::new(),
            symbols: RuntimeSymbolTable::default(),
        }
    }

    pub(crate) fn next_transaction_id(&self) -> crate::transactions::data::TransactionId {
        self.sequence.next_transaction_id()
    }

    pub(crate) fn next_proposal_ordinal(&self) -> Option<u64> {
        self.sequence.next_proposal_ordinal()
    }

    pub(crate) fn origin_identity(&self) -> sequence::origin::RelationalRuntimeOrigin {
        self.sequence.origin()
    }

    pub(crate) fn runtime_instance_id(&self) -> u64 {
        self.sequence.runtime_instance_id()
    }
}

impl RuntimeSubsystem for RuntimeServices {
    type Config = ();

    fn new(_: &Self::Config) -> Self {
        Self::empty()
    }

    fn fork(&self) -> Self {
        Self {
            sequence: RuntimeSequenceState::new(),
            instrumentation: self.instrumentation.fork(),
            symbols: self.symbols.detached_owner_snapshot(),
        }
    }
}
pub use symbol_table::RelationalSymbolTableForTest;

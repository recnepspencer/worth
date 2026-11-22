//! Owned refusal and last-owner custody for the new material receiver.
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use crate::ports::ManagedIndexCausePort;
use std::sync::Arc;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as ExecutionCause;

pub mod causes;
pub use causes::{WorthQueryManagedIndexDenialCause, WorthQueryManagedIndexRefusal};

/// A failed managed-index operation with its exact owned cause and material custody.
pub struct WorthQueryManagedIndexDenial {
    cause: Box<dyn ManagedIndexCausePort>,
    // Owned diagnostic backing drops before the actual original grant.
    _allocation: Option<Arc<SourceFactsAllocation>>,
}

impl WorthQueryManagedIndexDenial {
    pub fn from_cause(
        cause: Box<dyn ManagedIndexCausePort>,
        allocation: Option<Arc<SourceFactsAllocation>>,
    ) -> Self {
        Self {
            cause,
            _allocation: allocation,
        }
    }

    /// Borrows the original typed cause; no session or root authority is exposed.
    pub fn cause(&self) -> WorthQueryManagedIndexDenialCause<'_> {
        self.cause.cause()
    }

    pub fn retain(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, ExecutionCause> {
        let allocation = match &self._allocation {
            Some(value) => Arc::clone(value),
            None => SourceFactsAllocation::open(execution)?,
        };
        allocation.claim_arc::<Self>(execution)?;
        self._allocation = Some(allocation);
        Ok(Arc::new(self))
    }
}

impl std::fmt::Debug for WorthQueryManagedIndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ManagedIndexDenial")
            .field(&self.cause())
            .finish()
    }
}

impl std::fmt::Display for WorthQueryManagedIndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "managed index refused: {:?}", self.cause())
    }
}

impl std::error::Error for WorthQueryManagedIndexDenial {}

impl PartialEq for WorthQueryManagedIndexDenial {
    fn eq(&self, other: &Self) -> bool {
        self.cause() == other.cause()
    }
}

impl Eq for WorthQueryManagedIndexDenial {}

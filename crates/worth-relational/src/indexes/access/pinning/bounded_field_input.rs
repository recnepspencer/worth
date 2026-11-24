use super::IndexAccess;
use crate::indexes::data::{
    BoundedEntityFieldLookupDenial, BoundedEntityFieldLookupOutcome,
    BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};

pub(in crate::indexes::access) mod sealed {
    pub trait Sealed {}
}

/// A sealed exact read input. Snapshot and selected-view inputs preserve their
/// existing caller accounting; neither input retains execution custody.
pub trait BoundedEntityFieldLookupRead: sealed::Sealed {
    type Denial;
    #[doc(hidden)]
    fn execute(
        self,
        access: &IndexAccess<'_>,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Self::Denial>;
}

impl IndexAccess<'_> {
    /// The one bounded field lookup door; the caller always lends its request.
    pub fn execute_bounded_entity_field_lookup<Read: BoundedEntityFieldLookupRead>(
        &self,
        request: Read,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Read::Denial> {
        request.execute(self, parity, execution)
    }
}

impl sealed::Sealed for BoundedEntityFieldLookupRequest {}
impl BoundedEntityFieldLookupRead for BoundedEntityFieldLookupRequest {
    type Denial = BoundedEntityFieldLookupDenial;
    fn execute(
        self,
        access: &IndexAccess<'_>,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Self::Denial> {
        access.lookup_snapshot(self, parity_mode, execution)
    }
}

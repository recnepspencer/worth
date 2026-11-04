//! Sealed test doors over this module's private members.
use super::*;

impl StructuralWalkObservation {
    #[doc(hidden)]
    pub fn read_for_test(
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
        query: StructuralWalkQuery<'_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Arc<Self>, PinnedStructuralWalkDenial> {
        Self::read(pin, execution, query, context)
    }
}

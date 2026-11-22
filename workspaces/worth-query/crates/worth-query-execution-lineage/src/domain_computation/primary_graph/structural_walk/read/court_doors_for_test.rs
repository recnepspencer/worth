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

impl<'a> StructuralWalkQuery<'a> {
    #[doc(hidden)]
    pub fn new_for_test(
        relation: &'a str,
        roots: &'a [EntityId],
        entity_kind: KindId,
        relation_kind: KindId,
        direction: RelationDirection,
        maximum_work: Option<usize>,
        control: Option<&'a dyn Fn() -> Result<(), Cause>>,
    ) -> Self {
        Self {
            relation,
            roots,
            entity_kind,
            relation_kind,
            direction,
            maximum_work,
            control,
        }
    }
}

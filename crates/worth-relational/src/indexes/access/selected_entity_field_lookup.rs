use super::{
    bounded_entity_field_lookup::{sealed, BoundedEntityFieldLookupRead},
    IndexAccess,
};
use crate::identity::data::KindId;
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial as Denial,
    BoundedEntityFieldLookupDenialKind as Kind, BoundedEntityFieldLookupOutcome,
    BoundedIndexParityMode, DerivedIndexId, SelectedIndexReadWork,
};
use crate::runtime::VisibilityProjectionView;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

/// An issued selected view with its caller's existing admission callback.
/// The request is lent to the lookup, and is never kept in this input.
pub struct BoundedEntityFieldSelectedLookup<'a, 'runtime, E, F> {
    view: &'a VisibilityProjectionView<'runtime>,
    index: DerivedIndexId,
    kind: KindId,
    locator: &'a AspectFieldLocator,
    value: &'a AspectValue,
    limit: usize,
    prepare: F,
    error: std::marker::PhantomData<fn() -> E>,
}
impl<'a, 'runtime, E, F> BoundedEntityFieldSelectedLookup<'a, 'runtime, E, F>
where
    F: FnMut(SelectedIndexReadWork, u64) -> Result<(), E>,
{
    pub fn new(
        view: &'a VisibilityProjectionView<'runtime>,
        index: DerivedIndexId,
        kind: KindId,
        locator: &'a AspectFieldLocator,
        value: &'a AspectValue,
        limit: usize,
        prepare: F,
    ) -> Self {
        Self {
            view,
            index,
            kind,
            locator,
            value,
            limit,
            prepare,
            error: std::marker::PhantomData,
        }
    }
}
impl<E, F> sealed::Sealed for BoundedEntityFieldSelectedLookup<'_, '_, E, F> {}
impl<E, F> BoundedEntityFieldLookupRead for BoundedEntityFieldSelectedLookup<'_, '_, E, F>
where
    F: FnMut(SelectedIndexReadWork, u64) -> Result<(), E>,
{
    type Denial = Stop<E>;
    fn execute(
        self,
        access: &IndexAccess<'_>,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Stop<E>> {
        crate::visibility::materialization::read_records::read_request::consult(execution)
            .map_err(|cause| Stop::Lookup(Denial::new(Kind::Execution(cause), self.index)))?;
        access.collect_admitted_entity_field_lookup(
            execution,
            self.view,
            self.index,
            self.kind,
            self.locator,
            self.value,
            self.limit,
            parity,
            self.prepare,
        )
    }
}

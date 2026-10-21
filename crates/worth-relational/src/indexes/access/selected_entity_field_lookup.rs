//! Lookup at an issued exact selected view under the original execution request.
use super::{
    bounded_entity_field_lookup::{sealed, BoundedEntityFieldLookupRead},
    IndexAccess,
};
use crate::identity::data::KindId;
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial as Denial,
    BoundedEntityFieldLookupDenialKind as Kind, BoundedEntityFieldLookupOutcome,
    BoundedIndexParityMode, DerivedIndexId,
};
use crate::runtime::VisibilityProjectionView;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
pub struct BoundedEntityFieldSelectedLookup<'a> {
    view: &'a VisibilityProjectionView,
    index: DerivedIndexId,
    kind: KindId,
    locator: &'a AspectFieldLocator,
    value: &'a AspectValue,
    limit: usize,
}
impl<'a> BoundedEntityFieldSelectedLookup<'a> {
    pub fn new(
        view: &'a VisibilityProjectionView,
        index: DerivedIndexId,
        kind: KindId,
        locator: &'a AspectFieldLocator,
        value: &'a AspectValue,
        limit: usize,
    ) -> Self {
        Self {
            view,
            index,
            kind,
            locator,
            value,
            limit,
        }
    }
}
impl sealed::Sealed for BoundedEntityFieldSelectedLookup<'_> {}
impl BoundedEntityFieldLookupRead for BoundedEntityFieldSelectedLookup<'_> {
    type Denial = Denial;
    fn execute(
        self,
        access: &IndexAccess<'_>,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Denial> {
        let prepared = access.prepare_entity_field_lookup(
            self.view,
            self.index,
            self.kind,
            self.locator,
            execution,
        )?;
        // The prepared reader polls this request itself. This input has no additional
        // reader-specific interruption latch, unlike a codec-backed read session.
        prepared
            .execute_prepared_entity_field_lookup(self.value, self.limit, parity, execution, || {
                Ok::<(), std::convert::Infallible>(())
            })
            .map_err(|stop| match stop {
                Stop::Lookup(denial) => denial,
                Stop::Admission(never) => match never {},
                Stop::ExactBasisRequired | Stop::AccountingOverflow => {
                    Denial::new(Kind::SnapshotUnavailable, self.index)
                }
            })
    }
}

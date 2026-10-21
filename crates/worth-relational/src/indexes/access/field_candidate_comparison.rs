//! Compare a selected native entity field at its exact issued view.
use crate::identity::data::{EntityId, KindId};
use crate::indexes::data::{
    BoundedEntityFieldLookupDenial as Denial, BoundedEntityFieldLookupDenialKind as Kind,
    DerivedIndexId,
};
use crate::runtime::VisibilityProjectionView;
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, AuthoritativeRecordAspectState,
    ContractValidatedAspectValueView,
};
pub(super) fn borrowed_matches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    index: DerivedIndexId,
    view: &VisibilityProjectionView,
    entity: EntityId,
    locator: &AspectFieldLocator,
    expected: &AspectValue,
) -> Result<Option<(EntityId, KindId, bool)>, Denial> {
    crate::execution::check_request(execution)
        .map_err(|cause| Denial::new(Kind::Execution(cause), index))?;
    view.with_exact_entity_state(entity, |metadata, state| {
        (
            metadata.entity_id,
            metadata.kind_id,
            state_matches(state, locator, expected),
        )
    })
    .map_err(|_| Denial::new(Kind::SnapshotUnavailable, index))
}
fn state_matches(
    state: Option<&AuthoritativeRecordAspectState>,
    locator: &AspectFieldLocator,
    expected: &AspectValue,
) -> bool {
    let Some(state) = state else {
        return false;
    };
    let Some(validated) = state.get(locator.aspect().aspect_key()) else {
        return false;
    };
    let actual = match validated.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value),
        ContractValidatedAspectValueView::Struct(value) => {
            let [field] = locator.field_path().fields() else {
                return false;
            };
            value.get(field)
        }
    };
    actual.is_some_and(|actual| actual == expected)
}

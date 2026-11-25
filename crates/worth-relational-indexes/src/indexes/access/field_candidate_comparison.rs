//! Compare a selected native entity field at its exact issued view.
use crate::branch::indexes::data::bounded_entity_field_lookup::{
    BoundedEntityFieldLookupDenial as Denial, BoundedEntityFieldLookupDenialKind as Kind,
    BoundedEntityFieldLookupDenialOwnerPort as _,
};
use crate::branch::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::indexes::data::definition::DerivedIndexId;
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
    crate::contracts::execution::request_custody::check_request(execution)
        .map_err(|cause| Denial::new(Kind::Execution(cause), index))?;
    view.read_exact_entity_state(execution, entity, |metadata, state| {
        (
            metadata.entity_id,
            metadata.kind_id,
            state_matches(state, locator, expected),
        )
    })
    .map_err(|denial| {
        Denial::new(
            match denial {
                crate::branch::visibility::materialization::read_records::projection::borrowed_records::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
                    Kind::SnapshotUnavailable
                }
                crate::branch::visibility::materialization::read_records::projection::borrowed_records::RelationalBorrowedRecordReadDenial::Execution(cause) => {
                    Kind::Execution(cause)
                }
            },
            index,
        )
    })
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

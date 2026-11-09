//! Read declared relation field DATA from the public authoritative aspect state.
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};
use worth_relational::facade::runtime::RelationReadRecord;
pub(super) fn label(record: &RelationReadRecord) -> Option<&AspectValue> {
    let state = record.authoritative_aspect_state.as_ref()?;
    let payload = state.get(&super::current_field_meanings::aspect_key("label"))?;
    match payload.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value),
        ContractValidatedAspectValueView::Struct(fields) => {
            fields.get(&super::current_field_meanings::field_key("label"))
        }
    }
}

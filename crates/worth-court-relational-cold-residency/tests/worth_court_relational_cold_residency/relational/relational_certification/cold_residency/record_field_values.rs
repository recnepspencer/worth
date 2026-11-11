//! Observe the declared fixture name through public authoritative record DATA.
use super::current_field_meanings::{aspect_key, field_key};
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};
use worth_relational::facade::runtime::EntityReadRecord;
pub(super) fn name_value(record: &EntityReadRecord) -> Option<&AspectValue> {
    let state = record.authoritative_aspect_state.as_ref()?;
    match state.get(&aspect_key("name"))?.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value),
        ContractValidatedAspectValueView::Struct(value) => value.get(&field_key("name")),
    }
}
pub(super) fn name(record: &EntityReadRecord) -> Option<&str> {
    let AspectValue::String(text) = name_value(record)? else {
        return None;
    };
    text.raw_text()
}

//! Observe the declared fixture name through public authoritative record DATA.
use super::current_field_meanings::{aspect_key, field_key};
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView, InternedString};
use worth_relational::facade::runtime::EntityReadRecord;
pub(super) fn name(record: &EntityReadRecord) -> Option<&str> {
    let state = record.authoritative_aspect_state.as_ref()?;
    let value = match state.get(&aspect_key("name"))?.view() {
        ContractValidatedAspectValueView::Scalar(value) => value,
        ContractValidatedAspectValueView::Struct(value) => value.get(&field_key("name"))?,
    };
    let AspectValue::String(InternedString::Raw(text)) = value else {
        return None;
    };
    Some(text)
}

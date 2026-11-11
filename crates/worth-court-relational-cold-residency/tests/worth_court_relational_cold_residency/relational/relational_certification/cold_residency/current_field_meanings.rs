//! Named field meaning used by the actual current-read publications.
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_testkit_relational::durable_publication::field_patch;
pub(super) fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).unwrap()
}
pub(super) fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value).unwrap()
}
pub(super) fn name_field_patch(value: &str) -> AspectFieldPatch {
    field_patch("name", "name", value)
}
pub(super) fn relation_label_field_patch(value: &str) -> AspectFieldPatch {
    field_patch("label", "label", value)
}

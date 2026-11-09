//! A typed content request, checked against the record's installed format.
use super::ApplicationContentBinding;
use crate::application_schema::fragment::RecordSelection;
use std::marker::PhantomData;

pub struct ApplicationContentFieldRef<Field: RecordSelection, C> {
    field: Field,
    format: PhantomData<fn() -> C>,
}
impl<Field: RecordSelection, C> Copy for ApplicationContentFieldRef<Field, C> {}
impl<Field: RecordSelection, C> Clone for ApplicationContentFieldRef<Field, C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Field: RecordSelection, C> ApplicationContentFieldRef<Field, C> {
    pub fn field(&self) -> Field {
        self.field
    }
}
/// Requests an interpretation without registering a second storage definition.
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::{*, fragment::RecordSelection};
/// fn cannot_reinterpret<F:RecordSelection<Value=u64>, C:ApplicationContentBinding>(field:F, format:C) {
///     let _ = content_field(field, format);
/// }
/// ```
pub fn content_field<Field, C>(field: Field, _format: C) -> ApplicationContentFieldRef<Field, C>
where
    Field: RecordSelection,
    Field::Binding: super::ApplicationContentReferenceBinding,
    C: ApplicationContentBinding,
{
    ApplicationContentFieldRef {
        field,
        format: PhantomData,
    }
}

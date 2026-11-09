//! Typed resolution checks the record's exact installed value codec.
use super::*;
impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    pub(in crate::domain_computation::primary_graph) fn record_selection<
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection,
    >(
        &self,
        field: Field,
    ) -> Result<
        &worth_query_declaration::facade::application_schema::fragment::RecordFieldDefinition,
        WorthQueryEntityResolutionDenial,
    > {
        field
            .installed(
                self.application()
                    .installed_schema()
                    .value_bindings()
                    .fragments(),
            )
            .map_err(|_| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::FieldNotInstalled,
                    ),
                    field.field(),
                )
            })
    }
}

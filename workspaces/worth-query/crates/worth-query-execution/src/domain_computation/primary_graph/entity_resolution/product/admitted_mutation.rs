//! Selected conventional mutation scope through the installed equality field.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn resolve_entity_admitted_with_native_access<
        Field: RecordSelection,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: Field,
        value: Field::Value,
        request: &WorthQueryRequestScope,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        WorthQueryApplicationEntityIdentity<Schema, Field::Entity>,
        WorthQueryEntityResolutionDenial,
    > {
        admit_request(request, field.field())?;
        let column = self.record_selection(field)?;
        let locus = column.recipe().locus();
        // The declared field encoder is application-authored work. Its returned
        // value is moved through the framework lookup and identity construction.
        Field::Binding::validate(&value).map_err(|_| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                column.path(),
            )
        })?;
        let encoded = Field::Binding::encode(&value).map_err(|_| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                column.path(),
            )
        })?;
        let graph = self.application().runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::PrimaryGraphNotInstalled,
                ),
                field.entity(),
            )
        })?;

        let installed_field = graph
            .layout
            .equality_field(field.entity(), locus.aspect(), field.field())
            .ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::FieldNotInstalled,
                    ),
                    field.field(),
                )
            })?;
        let index = installed_field.equality_index_id.ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::FieldNotInstalled),
                field.field(),
            )
        })?;

        self.resolve_encoded_entity_admitted_with_native_access(
            execution,
            field.entity(),
            field.field(),
            encoded,
            request,
            index,
            installed_field,
            native,
        )
    }
}

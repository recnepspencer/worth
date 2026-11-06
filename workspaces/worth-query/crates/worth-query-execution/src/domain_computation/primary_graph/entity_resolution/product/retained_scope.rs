use super::*;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn resolve_retained_query_scope<
        Field: RecordSelection,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: Field,
        selector: &WorthQueryObservedScopeSelector,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationEntityIdentity<Schema, Field::Entity>,
        WorthQueryEntityResolutionDenial,
    > {
        self.resolve_retained_query_scope_with_native_access(execution, field, selector, request,
            &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_retained_query_scope_with_native_access<
        Field: RecordSelection,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: Field,
        selector: &WorthQueryObservedScopeSelector,
        request: &WorthQueryRequestScope,
        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        WorthQueryApplicationEntityIdentity<Schema, Field::Entity>,
        WorthQueryEntityResolutionDenial,
    > {
        // Every terminal refusal in this admitted route copies either the
        // entity or field name. Fund that one possible copy before validation.

        admit_request(request, field.field())?;
        let column = self.record_selection(field)?;
        let locus = column.recipe().locus();
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
        let installed_locator = &installed_field.locator;
        let index = installed_field.equality_index_id.ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::FieldNotInstalled),
                field.field(),
            )
        })?;

        // The first selected walk measures initialized comparison text; the
        // second installed walk measures the later locator-clone backing.

        if installed_locator != selector.locator() {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::FieldNotInstalled),
                field.field(),
            ));
        }
        if selector.value().value_family() != Field::Binding::SCALAR_FAMILY {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
            ));
        }

        self.resolve_encoded_entity_admitted_with_native_access(
            execution,
            field.entity(),
            field.field(),
            selector.value().clone(),
            request,
            index,
            installed_field,
            native,
        )
    }
}

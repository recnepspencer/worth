use super::*;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn resolve_retained_query_scope<Field: RecordSelection>(
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

    pub fn resolve_retained_query_scope_with_native_access<Field: RecordSelection>(
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
        self.resolve_query_scope_value_with_native_access(
            execution,
            field,
            Some(selector.locator()),
            selector.value(),
            request,
            native,
        )
    }

    /// Resolve the installed retained selector on the actual candidate image.
    /// The selector is input to the issuer, never authority for its subject.
    pub fn resolve_retained_applied_query_scope<Field: RecordSelection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: Field,
        selector: &WorthQueryObservedScopeSelector,
        before: &worth_relational::facade::runtime::RelationalPinnedRead,
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationEntityIdentity<Schema, Field::Entity>,
        WorthQueryEntityResolutionDenial,
    > {
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
        let installed = graph
            .layout_ref()
            .equality_field(field.entity(), locus.aspect(), field.field())
            .ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::FieldNotInstalled,
                    ),
                    field.field(),
                )
            })?;
        if &installed.locator != selector.locator()
            || selector.value().value_family() != Field::Binding::SCALAR_FAMILY
        {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
            ));
        }
        graph
            .retain_entity_resolution_context()
            .resolve_applied_identity(
                execution,
                before,
                candidate,
                field.entity(),
                locus.aspect(),
                field.field(),
                selector.value(),
            )
    }

    /// Both retained source readmission and declared coupled roots resolve the
    /// same installed equality field through this one canonical Native lookup.
    pub fn resolve_query_scope_value_with_native_access<Field: RecordSelection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: Field,
        locator: Option<&worth_foundational::facade::AspectFieldLocator>,
        value: &worth_foundational::facade::AspectValue,
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
            .layout_ref()
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

        if locator.is_some_and(|locator| installed_locator != locator) {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::FieldNotInstalled),
                field.field(),
            ));
        }
        if value.value_family() != Field::Binding::SCALAR_FAMILY {
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
            value.clone(),
            request,
            index,
            installed_field,
            native,
        )
    }
}

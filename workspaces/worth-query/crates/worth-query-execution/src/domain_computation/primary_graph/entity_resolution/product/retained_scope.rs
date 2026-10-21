use super::*;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn resolve_retained_query_scope<
        Aspect,
        Entity,
        Field,
        Value,
        Write,
        Unit,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        selector: &WorthQueryObservedScopeSelector,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        // Every terminal refusal in this admitted route copies either the
        // entity or field name. Fund that one possible copy before validation.

        admit_request(request, field.field())?;
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
            .equality_field(field.entity(), field.aspect(), field.field())
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

        self.resolve_encoded_entity_admitted(
            execution,
            field.entity(),
            field.field(),
            selector.value().clone(),
            request,
            index,
            installed_field,
        )
    }
}

//! Selected conventional mutation scope through the installed equality field.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn resolve_entity_admitted_with_native_access<
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
        value: Value,
        request: &WorthQueryRequestScope,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        admit_request(request, field.field())?;
        // The declared field encoder is application-authored work. Its returned
        // value is moved through the framework lookup and identity construction.
        let encoded = Field::Binding::encode(&value).map_err(|_| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
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
            .equality_field(field.entity(), field.aspect(), field.field())
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

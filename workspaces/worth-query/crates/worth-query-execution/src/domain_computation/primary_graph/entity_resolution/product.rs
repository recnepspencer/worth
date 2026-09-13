use super::{
    admit_request, entity_denial, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationScalarValueBinding, ApplicationSchema, EqualityPredicate,
    WorthQueryApplicationEntityIdentity, WorthQueryEntityResolutionDenial,
    WorthQueryEntityResolutionDenialKind, WorthQueryPrincipalResolutionMode,
    WorthQueryRequestScope, WritePosture,
};
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_installation::facade::DeclaredApplicationFieldValue;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn resolve_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &self,
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
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        admit_request(request, field.field())?;
        let encoded = Field::Binding::encode(&value).map_err(|_| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::ValueEncodingRejected,
                field.field(),
            )
        })?;
        let application = self.application();
        let graph = application.runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::PrimaryGraphNotInstalled,
                field.entity(),
            )
        })?;
        let installed = graph.retain_entity_resolution_context();
        let handle = graph.integration_handle();
        let result = handle.with_runtime_mut(|relational| {
            handle
                .ensure_primary_indexes_for_basis(relational, self.product().relational_basis())
                .map_err(|_| {
                    entity_denial(
                        WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                        field.field(),
                    )
                })?;
            installed
                .at_snapshot(relational, self.application_basis().snapshot_handle(), mode)
                .and_then(|truth| {
                    truth.resolve(field.entity(), field.aspect(), field.field(), encoded)
                })
        })?;
        admit_request(request, field.field())?;
        Ok(result.into_application_identity())
    }
}

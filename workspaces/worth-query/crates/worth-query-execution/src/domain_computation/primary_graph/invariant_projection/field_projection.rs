//! field projection.
use super::*;

impl<Schema> WorthQueryApplicationInvariantProjectionSnapshot<Schema>
where
    Schema: ApplicationSchema,
{
    /// Returns `None` for an unavailable identity or value, and preserves native
    /// request failure separately. The retained snapshot has no closure veto.
    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<Option<Value>, WorthQueryInvariantProjectionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if identity.authority_identity != self.authority_identity
            || identity.entity.as_ref() != field.entity()
        {
            return Ok(None);
        }
        let Some(locator) = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
        else {
            return Ok(None);
        };
        self.graph
            .with_runtime(execution, |runtime| {
                super::super::application_attempt::observe_field_value_borrowed(
                    runtime,
                    execution,
                    self.snapshot(),
                    identity.entity_id,
                    identity.kind,
                    &locator,
                    |value| value.and_then(|value| Field::Binding::decode(value).ok()),
                )
            })
            .map(Option::flatten)
            .map_err(|cause| {
                WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None)
            })
    }
}

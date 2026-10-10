//! field projection.
use super::*;

impl<Schema> WorthQueryApplicationInvariantProjectionSnapshot<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Option<Value>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if identity.authority_identity != self.authority_identity
            || identity.entity.as_ref() != field.entity()
        {
            return None;
        }
        let locator = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())?
            .clone();
        self.graph
            .with_runtime(execution, |runtime| {
                super::super::application_attempt::observe_field_value(
                    runtime,
                    execution,
                    self.snapshot(),
                    identity.entity_id,
                    identity.kind,
                    &locator,
                )
            })
            .and_then(|value| Field::Binding::decode(&value).ok())
    }
}

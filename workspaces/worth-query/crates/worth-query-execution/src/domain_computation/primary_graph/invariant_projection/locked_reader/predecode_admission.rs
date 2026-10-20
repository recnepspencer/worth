//! Borrow one exact snapshot scalar without the ordinary observation's clone.
use super::WorthQueryApplicationInvariantProjectionReader;
use crate::domain_computation::primary_graph::{
    application_attempt::observe_field_value_borrowed_admitted, HandlerExecutionDenial,
    HandlerInterruption, WorthQueryInvariantEntityIdentity,
};
use worth_foundational::facade::AspectValue as ApplicationValue;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue, WritePosture,
};

#[path = "predecode_admission/decoding.rs"]
mod decoding;

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::type_complexity)]
    pub(in crate::domain_computation::primary_graph) fn field_with_predecode_admission<
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
        Denied,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        admit: impl for<'raw, 'checkpoint> FnOnce(
            &'raw ApplicationValue,
            &'checkpoint dyn Fn() -> Result<(), HandlerInterruption>,
        ) -> Result<(), Denied>,
        checkpoint: &dyn Fn() -> Result<(), HandlerInterruption>,
    ) -> Result<Result<Option<Value>, Denied>, HandlerExecutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if !self.identity_is_local(identity, field.entity()) || !self.work_budget.can_afford(1) {
            return Ok(Ok(None));
        }
        assert!(self.work_budget.consume(1), "projection work was admitted");
        self.realized_scope.record(identity.entity_id);
        let Some(locator) = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
        else {
            return Ok(Ok(None));
        };
        self.work.record_field();
        let observed = observe_field_value_borrowed_admitted(
            self.runtime,
            self.execution,
            self.snapshot,
            identity.entity_id,
            identity.kind,
            &locator,
            |raw| {
                let Some(raw) = raw else { return Ok(Ok(None)) };
                decoding::decode_admitted::<Field::Binding, Denied>(raw, admit, checkpoint)
            },
        );
        match observed {
            Ok(value) => value.unwrap_or(Ok(Ok(None))),
            Err(cause) => {
                self.native_read_stop.get_or_insert(cause);
                Err(HandlerExecutionDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::from(cause),
                ))
            }
        }
    }
}

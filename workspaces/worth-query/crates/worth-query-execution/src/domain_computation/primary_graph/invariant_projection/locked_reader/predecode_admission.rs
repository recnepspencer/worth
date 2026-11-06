//! Borrow one exact snapshot scalar without the ordinary observation's clone.
use super::WorthQueryApplicationInvariantProjectionReader;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, HandlerInterruption, WorthQueryInvariantEntityIdentity,
};
use worth_foundational::facade::AspectValue as ApplicationValue;
use worth_query_installation::facade::{ApplicationReadableScalarValueBinding, ApplicationSchema};

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
        Denied,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
        admit: impl for<'raw, 'checkpoint> FnOnce(
            &'raw ApplicationValue,
            &'checkpoint dyn Fn() -> Result<(), HandlerInterruption>,
        ) -> Result<(), Denied>,
        checkpoint: &dyn Fn() -> Result<(), HandlerInterruption>,
    ) -> Result<Result<Option<Value>, Denied>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        if field.installed(self.value_bindings.fragments()).is_err() {
            self.value_decode.refuse_content(crate::domain_computation::primary_graph::WorthQueryValueDecodeDenial::BindingContractMismatch);
            return Ok(Ok(None));
        }
        if self.native_read_stop.is_some()
            || self.value_decode.stopped()
            || !self.identity_is_local(identity, field.entity())
            || !self.work_budget.can_afford(1)
        {
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
        let reads = &mut self.work;
        let decoder = self.value_decode;
        let observed = self.native.observe_field(
            self.execution,
            self.snapshot,
            identity.entity_id,
            identity.kind,
            &locator,
            |raw| {
                reads.record_fields(1);
                let Some(raw) = raw.value() else {
                    return Ok(Ok(None));
                };
                decoding::decode_admitted(raw, admit, checkpoint, |raw| {
                    decoder.decode::<Field::Binding>(
                        field.entity(),
                        field.aspect(),
                        field.field(),
                        raw,
                    )
                })
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

#[path = "predecode_admission_court_doors_for_test.rs"]
mod court_doors_for_test;

//! Retained native pin and original-request custody through one field-read handoff.
use super::super::value_decode::{ValueDecodeSession, ValueDecodeState};
use super::*;
use std::cell::Cell;
use worth_relational::facade::transactions::RelationalExecutionDenialCause;

/// One request's reads of an existing retained snapshot. A swallowed field
/// refusal vetoes the closure's output; the native pin remains borrowed.
pub struct WorthQueryRetainedSnapshotRead<'read, 'request, 'authority, Schema> {
    snapshot: &'read WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
    execution: &'read worth_execution::ExecutionRequest<'request, 'authority>,
    decode: &'read ValueDecodeSession<'read>,
    native_stop: Cell<Option<RelationalExecutionDenialCause>>,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionSnapshot<Schema> {
    /// Complete several field reads under one original allowance. Decode memory
    /// remains reserved until the whole output is accepted or discarded.
    pub fn read_fields<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&WorthQueryRetainedSnapshotRead<'_, '_, '_, Schema>) -> Output,
    ) -> Result<Output, WorthQueryInvariantProjectionDenial> {
        let decode = ValueDecodeState::new(self.value_bindings.clone());
        let (output, native_stop) = decode
            .run(execution, |session| {
                let reader = WorthQueryRetainedSnapshotRead {
                    snapshot: self,
                    execution,
                    decode: session,
                    native_stop: Cell::new(None),
                };
                let output = read(&reader);
                (output, reader.native_stop.get())
            })
            .map_err(|cause| {
                WorthQueryInvariantProjectionDenial::value_decode_stopped(cause, None)
            })?;
        if let Some(cause) = native_stop {
            return Err(
                WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None),
            );
        }
        decode.check(execution).map_err(|cause| {
            WorthQueryInvariantProjectionDenial::value_decode_stopped(cause, None)
        })?;
        Ok(output)
    }

    /// One field is one completed read. Use `read_fields` for a multi-read
    /// decision whose cumulative custody must survive until its final handoff.
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
        self.read_fields(execution, |read| read.field(identity, field))?
    }
}

impl<Schema: ApplicationSchema> WorthQueryRetainedSnapshotRead<'_, '_, '_, Schema> {
    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<Option<Value>, WorthQueryInvariantProjectionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if let Some(cause) = self.native_stop.get() {
            return Err(
                WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None),
            );
        }
        if let Some(cause) = self.decode.denial() {
            return Err(WorthQueryInvariantProjectionDenial::value_decode_stopped(
                cause, None,
            ));
        }
        let snapshot = self.snapshot;
        if identity.authority_identity != snapshot.authority_identity
            || identity.entity.as_ref() != field.entity()
        {
            return Ok(None);
        }
        let Some(locator) = snapshot
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
        else {
            return Ok(None);
        };
        let observed = snapshot.graph.with_runtime(self.execution, |runtime| {
            super::super::application_attempt::observe_field_value_borrowed(
                runtime,
                self.execution,
                snapshot.snapshot(),
                identity.entity_id,
                identity.kind,
                &locator,
                |value| {
                    value.and_then(|value| {
                        self.decode.decode::<Field::Binding>(
                            field.entity(),
                            field.aspect(),
                            field.field(),
                            value,
                        )
                    })
                },
            )
        });
        if let Some(cause) = self.decode.denial() {
            return Err(WorthQueryInvariantProjectionDenial::value_decode_stopped(
                cause, None,
            ));
        }
        observed.map(Option::flatten).map_err(|cause| {
            self.native_stop.set(Some(cause));
            WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None)
        })
    }
}

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
            WorthQueryInvariantProjectionDenial::value_decode_handoff_stopped(cause, None)
        })?;
        Ok(output)
    }

    /// One field is one completed read. Use `read_fields` for a multi-read
    /// decision whose cumulative custody must survive until its final handoff.
    pub fn field<Entity, Aspect, Field, Value>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
    ) -> Result<Option<Value>, WorthQueryInvariantProjectionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let mut decoded = super::super::value_decode::ScalarFieldDecoder::<Field::Binding>::new();
        self.field_core().one(
            &self.value_bindings,
            execution,
            identity.native_identity(),
            field.identity(),
            &mut decoded,
        )?;
        Ok(decoded.value)
    }
}

impl<Schema: ApplicationSchema> WorthQueryRetainedSnapshotRead<'_, '_, '_, Schema> {
    pub fn field<Entity, Aspect, Field, Value>(
        &self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
    ) -> Result<Option<Value>, WorthQueryInvariantProjectionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let mut decoded = super::super::value_decode::ScalarFieldDecoder::<Field::Binding>::new();
        self.snapshot.field_core().field(
            self.execution,
            self.decode,
            &self.native_stop,
            identity.native_identity(),
            field.identity(),
            &mut decoded,
        )?;
        Ok(decoded.value)
    }
}

struct SnapshotFieldCore<'a> {
    graph: &'a super::super::WorthQueryPrimaryGraphIntegrationHandle,
    layout: &'a super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    snapshot: &'a worth_relational::facade::snapshots::SnapshotHandle,
    authority: u64,
}
impl<S: ApplicationSchema> WorthQueryApplicationInvariantProjectionSnapshot<S> {
    #[inline(always)]
    fn field_core(&self) -> SnapshotFieldCore<'_> {
        SnapshotFieldCore {
            graph: &self.graph,
            layout: &self.layout,
            snapshot: self.snapshot(),
            authority: self.authority_identity,
        }
    }
}
impl SnapshotFieldCore<'_> {
    fn one(
        &self,
        bindings:&worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: super::native_read::NativeIdentity<'_>,
        field: worth_query_declaration::facade::application_schema::fragment::ColumnIdentity,
        decoded: &mut dyn super::super::value_decode::FieldDecoder,
    ) -> Result<(), WorthQueryInvariantProjectionDenial> {
        let decode = ValueDecodeState::new(bindings.clone());
        let (answer, stop) = decode
            .run(execution, |session| {
                let stopped = Cell::new(None);
                let answer = self.field(execution, session, &stopped, identity, field, decoded);
                (answer, stopped.get())
            })
            .map_err(|cause| {
                WorthQueryInvariantProjectionDenial::value_decode_stopped(cause, None)
            })?;
        if let Some(cause) = stop {
            return Err(
                WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None),
            );
        }
        decode.check(execution).map_err(|cause| {
            WorthQueryInvariantProjectionDenial::value_decode_handoff_stopped(cause, None)
        })?;
        answer
    }
    fn field(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        decode: &ValueDecodeSession<'_>,
        native_stop: &Cell<Option<RelationalExecutionDenialCause>>,
        identity: super::native_read::NativeIdentity<'_>,
        field: worth_query_declaration::facade::application_schema::fragment::ColumnIdentity,
        decoded: &mut dyn super::super::value_decode::FieldDecoder,
    ) -> Result<(), WorthQueryInvariantProjectionDenial> {
        if let Some(cause) = native_stop.get() {
            return Err(
                WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None),
            );
        }
        if let Some(cause) = decode.denial() {
            return Err(WorthQueryInvariantProjectionDenial::value_decode_stopped(
                cause, None,
            ));
        }
        if identity.authority != self.authority || identity.entity.as_ref() != field.entity() {
            return Ok(());
        }
        let Some(locator) = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
        else {
            return Ok(());
        };
        let observed = self.graph.with_runtime(execution, |runtime| {
            super::super::application_attempt::observe_field_value_borrowed(
                runtime,
                execution,
                self.snapshot,
                identity.entity_id,
                identity.kind,
                &locator,
                |raw| {
                    if let Some(raw) = raw {
                        decoded.decode(decode, field.entity(), field.aspect(), field.field(), raw);
                    }
                },
            )
        });
        if let Some(cause) = decode.denial() {
            return Err(WorthQueryInvariantProjectionDenial::value_decode_stopped(
                cause, None,
            ));
        }
        observed.map(|_| ()).map_err(|cause| {
            native_stop.set(Some(cause));
            WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None)
        })
    }
}

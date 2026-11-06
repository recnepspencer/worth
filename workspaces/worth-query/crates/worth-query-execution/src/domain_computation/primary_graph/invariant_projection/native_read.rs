//! One concrete snapshot read path; typed callers lend identities and codec slots.
use super::*;
mod entity;
mod field_observation;
mod relation_field;
mod traversal;
pub(super) use entity::{typed_optional, NativeEntity};
mod walk;
use super::super::value_decode::FieldDecoder;
use super::work::WorthQueryInvariantProjectionWorkBudget;
pub(super) use walk::TypedRoots;
use worth_query_declaration::facade::application_schema::fragment::ColumnIdentity;
use worth_relational::facade::{
    runtime::RelationalRuntime, snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause,
};

#[derive(Clone, Copy)]
pub(super) struct NativeIdentity<'a> {
    pub entity: &'a Arc<str>,
    pub entity_id: EntityId,
    pub kind: KindId,
    pub authority: u64,
}
pub(super) struct NativeRead<'a, 'r> {
    pub runtime: &'a mut RelationalRuntime,
    pub execution: &'r worth_execution::ExecutionRequest<'r, 'r>,
    pub snapshot: &'r SnapshotHandle,
    pub layout: &'r WorthQueryPrimaryGraphLayout,
    pub bindings:
        &'r worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    pub work: &'a mut WorthQueryInvariantProjectionWork,
    pub budget: &'a mut WorthQueryInvariantProjectionWorkBudget,
    pub stopped: &'a mut Option<RelationalExecutionDenialCause>,
    pub scope: &'a mut WorthQueryRealizedProjectionScope,
    pub decoder: &'r super::super::value_decode::ValueDecodeSession<'r>,
    pub authority: u64,
}
impl<'r, S> WorthQueryApplicationInvariantProjectionReader<'r, S> {
    #[inline(always)]
    pub(super) fn native_read(&mut self) -> NativeRead<'_, 'r> {
        NativeRead {
            runtime: self.runtime,
            execution: self.execution,
            snapshot: self.snapshot,
            layout: self.layout,
            bindings: self.value_bindings,
            work: &mut self.work,
            budget: &mut self.work_budget,
            stopped: &mut self.native_read_stop,
            scope: &mut self.realized_scope,
            decoder: self.value_decode,
            authority: self.authority_identity,
        }
    }
}
impl<S, E> WorthQueryInvariantEntityIdentity<S, E> {
    #[inline(always)]
    pub(super) fn native_identity(&self) -> NativeIdentity<'_> {
        NativeIdentity {
            entity: &self.entity,
            entity_id: self.entity_id,
            kind: self.kind,
            authority: self.authority_identity,
        }
    }
}
impl NativeRead<'_, '_> {
    pub fn local(&self, identity: NativeIdentity<'_>, entity: &str) -> bool {
        identity.authority == self.authority && identity.entity.as_ref() == entity
    }
    /// One field lookup and one physical read; no decoded-value boxing or copying.
    pub fn field(
        &mut self,
        identity: NativeIdentity<'_>,
        field: ColumnIdentity,
        decode: &mut dyn FieldDecoder,
    ) {
        if field.installed(self.bindings.fragments()).is_err() {
            self.decoder
                .refuse_content(WorthQueryValueDecodeDenial::BindingContractMismatch);
            return;
        }
        if self.stopped.is_some()
            || self.decoder.stopped()
            || !self.local(identity, field.entity())
            || !self.budget.can_afford(1)
        {
            return;
        }
        assert!(self.budget.consume(1), "projection work was admitted");
        self.scope.record(identity.entity_id);
        let Some(locator) = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
        else {
            return;
        };
        let reads = &mut self.work;
        let observed = super::super::application_attempt::observe_field_value_borrowed_admitted(
            self.runtime,
            self.execution,
            self.snapshot,
            identity.entity_id,
            identity.kind,
            &locator,
            |raw| {
                reads.record_fields(1);
                if let Some(raw) = raw {
                    decode.decode(
                        self.decoder,
                        field.entity(),
                        field.aspect(),
                        field.field(),
                        raw,
                    );
                }
            },
        );
        if let Err(cause) = observed {
            self.stopped.get_or_insert(cause);
            decode.discard();
        }
    }
}

impl WorthQueryInvariantEntityIdentity<(), ()> {
    #[inline(always)]
    pub(super) fn into_typed<S, E>(self) -> WorthQueryInvariantEntityIdentity<S, E> {
        WorthQueryInvariantEntityIdentity {
            entity_id: self.entity_id,
            kind: self.kind,
            entity: self.entity,
            authority_identity: self.authority_identity,
            _marker: PhantomData,
        }
    }
}
impl NativeIdentity<'_> {
    fn owned(self) -> WorthQueryInvariantEntityIdentity<(), ()> {
        WorthQueryInvariantEntityIdentity {
            entity_id: self.entity_id,
            kind: self.kind,
            entity: Arc::clone(self.entity),
            authority_identity: self.authority,
            _marker: PhantomData,
        }
    }
}
